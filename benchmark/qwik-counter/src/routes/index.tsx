import { component$, useSignal } from "@builder.io/qwik";
import type { DocumentHead } from "@builder.io/qwik-city";

export default component$(() => {
  const count = useSignal(0);

  return (
    <main>
      <h1>Qwik Counter</h1>
      <p>Current count: {count.value}</p>
      <button type="button" onClick$={() => count.value++}>
        +
      </button>
    </main>
  );
});

export const head: DocumentHead = {
  title: "Qwik Counter",
};
