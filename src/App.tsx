import { add } from "../rust/pkg/meal";
import "./App.css";

function App() {
  const result = add(60, 9);
  return <>{result}</>;
}

export default App;
