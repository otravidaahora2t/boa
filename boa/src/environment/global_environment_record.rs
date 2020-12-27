//! # Global Environment Records
//!
//! A global Environment Record is used to represent the outer most scope that is shared by all
//! of the ECMAScript Script elements that are processed in a common realm.
//! A global Environment Record provides the bindings for built-in globals (clause 18),
//! properties of the global object, and for all top-level declarations (13.2.8, 13.2.10)
//! that occur within a Script.
//! More info:  <https://tc39.es/ecma262/#sec-global-environment-records>

use super::ErrorKind;
use crate::{
    environment::{
        declarative_environment_record::DeclarativeEnvironmentRecord,
        environment_record_trait::EnvironmentRecordTrait,
        lexical_environment::{Environment, EnvironmentType},
        object_environment_record::ObjectEnvironmentRecord,
    },
    gc::{Finalize, Trace},
    property::{Attribute, DataDescriptor},
    Value,
};
use rustc_hash::FxHashSet;

#[derive(Debug, Trace, Finalize, Clone)]
pub struct GlobalEnvironmentRecord {
    pub object_record: ObjectEnvironmentRecord,
    pub global_this_binding: Value,
    pub declarative_record: DeclarativeEnvironmentRecord,
    pub var_names: FxHashSet<String>,
}

impl GlobalEnvironmentRecord {
    pub fn has_var_declaration(&self, name: &str) -> bool {
        self.var_names.contains(name)
    }

    pub fn has_lexical_declaration(&self, name: &str) -> bool {
        self.declarative_record.has_binding(name)
    }

    pub fn has_restricted_global_property(&self, name: &str) -> bool {
        let global_object = &self.object_record.bindings;
        let existing_prop = global_object.get_property(name);
        match existing_prop {
            Some(desc) => {
                if desc.configurable() {
                    return false;
                }
                true
            }
            None => false,
        }
    }

    pub fn can_declare_global_var(&self, name: &str) -> bool {
        let global_object = &self.object_record.bindings;
        if global_object.has_field(name) {
            true
        } else {
            global_object.is_extensible()
        }
    }

    pub fn can_declare_global_function(&self, name: &str) -> bool {
        let global_object = &self.object_record.bindings;
        let existing_prop = global_object.get_property(name);
        match existing_prop {
            Some(prop) => {
                if prop.configurable() {
                    true
                } else {
                    prop.is_data_descriptor() && prop.attributes().writable() && prop.enumerable()
                }
            }
            None => global_object.is_extensible(),
        }
    }

    pub fn create_global_var_binding(
        &mut self,
        name: String,
        deletion: bool,
    ) -> Result<(), ErrorKind> {
        let obj_rec = &mut self.object_record;
        let global_object = &obj_rec.bindings;
        let has_property = global_object.has_field(name.as_str());
        let extensible = global_object.is_extensible();
        if !has_property && extensible {
            obj_rec.create_mutable_binding(name.clone(), deletion)?;
            obj_rec.initialize_binding(&name, Value::undefined())?;
        }

        let var_declared_names = &mut self.var_names;
        if !var_declared_names.contains(&name) {
            var_declared_names.insert(name);
        }
        Ok(())
    }

    pub fn create_global_function_binding(&mut self, name: &str, value: Value, deletion: bool) {
        let global_object = &mut self.object_record.bindings;
        let existing_prop = global_object.get_property(name);
        let desc = match existing_prop {
            Some(desc) if desc.configurable() => DataDescriptor::new(value, Attribute::empty()),
            Some(_) => {
                let mut attributes = Attribute::WRITABLE | Attribute::ENUMERABLE;
                if deletion {
                    attributes |= Attribute::CONFIGURABLE;
                }
                DataDescriptor::new(value, attributes)
            }
            None => DataDescriptor::new(value, Attribute::empty()),
        };

        global_object
            .as_object()
            .expect("global object")
            .insert(name, desc);
    }
}

impl EnvironmentRecordTrait for GlobalEnvironmentRecord {
    fn get_this_binding(&self) -> Result<Value, ErrorKind> {
        Ok(self.global_this_binding.clone())
    }

    fn has_binding(&self, name: &str) -> bool {
        if self.declarative_record.has_binding(name) {
            return true;
        }
        self.object_record.has_binding(name)
    }

    fn create_mutable_binding(&mut self, name: String, deletion: bool) -> Result<(), ErrorKind> {
        if self.declarative_record.has_binding(&name) {
            return Err(ErrorKind::new_type_error(format!(
                "Binding already exists for {}",
                name
            )));
        }

        self.declarative_record
            .create_mutable_binding(name, deletion)
    }

    fn create_immutable_binding(&mut self, name: String, strict: bool) -> Result<(), ErrorKind> {
        if self.declarative_record.has_binding(&name) {
            return Err(ErrorKind::new_type_error(format!(
                "Binding already exists for {}",
                name
            )));
        }

        self.declarative_record
            .create_immutable_binding(name, strict)
    }

    fn initialize_binding(&mut self, name: &str, value: Value) -> Result<(), ErrorKind> {
        if self.declarative_record.has_binding(&name) {
            return self.declarative_record.initialize_binding(name, value);
        }

        assert!(
            self.object_record.has_binding(name),
            "Binding must be in object_record"
        );
        self.object_record.initialize_binding(name, value)
    }

    fn set_mutable_binding(
        &mut self,
        name: &str,
        value: Value,
        strict: bool,
    ) -> Result<(), ErrorKind> {
        if self.declarative_record.has_binding(&name) {
            return self
                .declarative_record
                .set_mutable_binding(name, value, strict);
        }
        self.object_record.set_mutable_binding(name, value, strict)
    }

    fn get_binding_value(&self, name: &str, strict: bool) -> Result<Value, ErrorKind> {
        if self.declarative_record.has_binding(&name) {
            return self.declarative_record.get_binding_value(name, strict);
        }
        self.object_record.get_binding_value(name, strict)
    }

    fn delete_binding(&mut self, name: &str) -> bool {
        if self.declarative_record.has_binding(&name) {
            return self.declarative_record.delete_binding(name);
        }

        let global: &Value = &self.object_record.bindings;
        if global.has_field(name) {
            let status = self.object_record.delete_binding(name);
            if status {
                let var_names = &mut self.var_names;
                if var_names.contains(name) {
                    var_names.remove(name);
                    return status;
                }
            }
        }
        true
    }

    fn has_this_binding(&self) -> bool {
        true
    }

    fn has_super_binding(&self) -> bool {
        false
    }

    fn with_base_object(&self) -> Value {
        Value::undefined()
    }

    fn get_outer_environment(&self) -> Option<Environment> {
        None
    }

    fn set_outer_environment(&mut self, _env: Environment) {
        // TODO: Implement
        panic!("Not implemented yet")
    }

    fn get_environment_type(&self) -> EnvironmentType {
        EnvironmentType::Global
    }

    fn get_global_object(&self) -> Option<Value> {
        Some(self.global_this_binding.clone())
    }
}
