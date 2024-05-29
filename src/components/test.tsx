import * as wasm from "../../rust/pkg/meal";
import { createResource } from "solid-js";
import { clientOnly } from "@solidjs/start";

export default function App() {
  // const initialization = createResource(init);
  const result = wasm.add(60, 9);
  return <>{result}</>;
}
