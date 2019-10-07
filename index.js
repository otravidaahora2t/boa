// Note that a dynamic `import` statement here is required due to
// webpack/webpack#6615, but in theory `import { greet } from './pkg/hello_world';`
// will work here one day as well!
const rust = import("./pkg");
// const image = import("./assets/01_rust_loves_js.png");

rust.then(m => {
  window.evaluate = m.evaluate;
  let textarea = document.querySelector("textarea");
  textarea.addEventListener("input", inputHandler);
});

function inputHandler(evt) {
  let text = evt.target.value;
  let p = document.querySelector("p.output");
  let t0 = performance.now();
  let result = window.evaluate(text);
  let t1 = performance.now();
  p.textContent = `> ${result}`;
  console.log(result);
}
