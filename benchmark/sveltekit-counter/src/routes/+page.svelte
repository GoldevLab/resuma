<script lang="ts">
  let count = $state(0);
</script>

<main>
  <h1>SvelteKit Counter</h1>
  <p>Current count: {count}</p>
  <button type="button" onclick={() => count++}>+</button>
</main>
