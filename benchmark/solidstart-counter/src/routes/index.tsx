import { createSignal } from "solid-js";

export default function Home() {
  const [count, setCount] = createSignal(0);

  return (
    <main>
      <h1>SolidStart Counter</h1>
      <p>Current count: {count()}</p>
      <button type="button" onClick={() => setCount((c) => c + 1)}>
        +
      </button>
    </main>
  );
}
