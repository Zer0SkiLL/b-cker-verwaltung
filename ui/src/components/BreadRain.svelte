<script lang="ts">
  // Osternest: kurzer Brot-Regen bei einem Meilenstein (bewusst nicht im Handbuch).
  // Klicks gehen durch (pointer-events: none); jeder Klick oder Tastendruck beendet ihn sofort.
  import { onDestroy } from 'svelte';
  import { rain, RAIN_MS } from '../lib/osternest/milestone';

  const BREADS = ['🥐', '🥨', '🍞'];
  const PIECES = 24;

  let timer: ReturnType<typeof setTimeout> | undefined;

  const pieces = $derived(
    $rain === null
      ? []
      : Array.from({ length: PIECES }, (_, i) => ({
          icon: BREADS[i % BREADS.length],
          left: Math.random() * 100,
          delay: Math.random() * 1.2,
        })),
  );

  function stop() {
    clearTimeout(timer);
    rain.set(null);
  }

  $effect(() => {
    if ($rain === null) return;
    timer = setTimeout(stop, RAIN_MS);
    return () => clearTimeout(timer);
  });

  onDestroy(() => clearTimeout(timer));
</script>

<svelte:window onpointerdown={() => $rain !== null && stop()} onkeydown={() => $rain !== null && stop()} />

{#if $rain !== null}
  <div class="rain" aria-live="polite">
    <div class="card">{$rain} aktive Mitglieder! 🥐</div>
    {#each pieces as piece, i (i)}
      <span class="piece" aria-hidden="true" style:left="{piece.left}%" style:animation-delay="{piece.delay}s">{piece.icon}</span>
    {/each}
  </div>
{/if}

<style>
  .rain { position: fixed; inset: 0; pointer-events: none; z-index: 70; overflow: hidden; }
  .card { position: absolute; top: 1rem; left: 50%; transform: translateX(-50%); background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 0.5rem 1rem; font-weight: 600; box-shadow: 0 4px 16px rgb(0 0 0 / 0.15); }
  .piece { position: absolute; top: -2rem; font-size: 1.8rem; animation: fall 1.8s linear forwards; }
  @keyframes fall { to { transform: translateY(110vh) rotate(240deg); } }
  @media (prefers-reduced-motion: reduce) { .piece { display: none; } }
</style>
