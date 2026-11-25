<script lang="ts">
  // Aufbewahrung der Sicherungen und Sicherung während der Arbeit (gilt für alle, die die Datei öffnen).
  import { onMount } from 'svelte';
  import * as api from '../../lib/api';
  import { openHelp } from '../../lib/help';
  import type { RetentionPolicy } from '../../lib/types';

  let { readOnly }: { readOnly: boolean } = $props();

  const DEFAULTS: RetentionPolicy = { keepAllDays: 2, dailyDays: 14, weeklyWeeks: 8, monthlyMonths: 12, intervalMinutes: 60 };
  const FIELDS: { key: keyof RetentionPolicy; label: string }[] = [
    { key: 'keepAllDays', label: 'Alle Sicherungen behalten für … Tage' },
    { key: 'dailyDays', label: 'Eine pro Tag für … Tage' },
    { key: 'weeklyWeeks', label: 'Eine pro Woche für … Wochen' },
    { key: 'monthlyMonths', label: 'Eine pro Monat für … Monate' },
    { key: 'intervalMinutes', label: 'Während der Arbeit sichern alle … Minuten (0 = aus)' },
  ];

  let policy = $state<RetentionPolicy | null>(null);
  let error = $state<string | null>(null);
  let info = $state<string | null>(null);

  onMount(async () => {
    try {
      policy = await api.getRetention();
    } catch (err) {
      error = (err as Error).message;
    }
  });

  function setField(key: keyof RetentionPolicy, value: string) {
    if (!policy) return;
    policy = { ...policy, [key]: value.trim() === '' ? NaN : Number(value) };
    info = null;
  }

  async function save() {
    if (!policy) return;
    error = null;
    // Leere Felder, Kommazahlen oder Negatives gar nicht erst senden.
    if (!FIELDS.every((f) => Number.isInteger(policy![f.key]) && policy![f.key] >= 0)) {
      error = 'Bitte in jedes Feld eine ganze Zahl ab 0 eingeben.';
      return;
    }
    try {
      await api.setRetention(policy);
      info = 'Gespeichert.';
    } catch (err) {
      error = (err as Error).message;
    }
  }
</script>

<h2>Aufbewahrung</h2>
<p class="muted">
  Ältere Sicherungen werden ausgedünnt: zuerst alle, danach eine pro Tag, Woche und Monat. Die neueste bleibt immer.
  <button class="link" onclick={() => openHelp('08-sicherung-wiederherstellen')}>So funktioniert die Aufbewahrung</button>
</p>
{#if error}<div class="form-error">{error}</div>{/if}
{#if policy}
  <div class="card grid">
    {#each FIELDS as f (f.key)}
      <label for="ret-{f.key}">{f.label}</label>
      <input
        id="ret-{f.key}"
        type="number"
        min="0"
        value={policy[f.key]}
        disabled={readOnly}
        oninput={(e) => setField(f.key, e.currentTarget.value)}
      />
    {/each}
    <div class="toolbar actions">
      <button class="primary" disabled={readOnly} onclick={save}>Speichern</button>
      <button disabled={readOnly} onclick={() => { policy = { ...DEFAULTS }; info = null; }}>Standard wiederherstellen</button>
      {#if info}<span class="muted">{info}</span>{/if}
    </div>
  </div>
{/if}

<style>
  .grid { display: grid; grid-template-columns: 1fr 7rem; gap: 0.4rem 1rem; align-items: center; max-width: 34rem; }
  .actions { grid-column: 1 / -1; margin-top: 0.4rem; }
</style>
