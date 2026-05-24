import { useState } from "react";

export default function Counter() {
  const [count, setCount] = useState(0);

  return (
    <main>
      <h1>Astro Counter</h1>
      <p>Current count: {count}</p>
      <button type="button" onClick={() => setCount((c) => c + 1)}>
        +
      </button>
    </main>
  );
}
