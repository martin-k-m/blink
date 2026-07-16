import { useState } from "react";

// This fixture also declares a date-formatting dependency in package.json
// that is deliberately never imported anywhere in this file, to exercise
// Blink's unused dependency detection.
export default function App() {
  const [count, setCount] = useState(0);
  return <button onClick={() => setCount((c) => c + 1)}>{count}</button>;
}
