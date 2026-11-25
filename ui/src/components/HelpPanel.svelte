<script lang="ts">
  import { onMount } from 'svelte';
  import { openWebsite } from '../lib/desktop';
  import { reportOpen } from '../lib/diagnostics';
  import { PARTS, pdfUrl, renderChapter, searchChapters, slugFromHref } from '../lib/handbook';
  import { closeHelp, helpBack, helpState, openHelp, showSearch, showToc } from '../lib/help';

  let version = $state('');
  let query = $state('');

  onMount(async () => {
    try {
      version = await (await import('@tauri-apps/api/app')).getVersion();
    } catch {
      version = '';
    }
  });

  const view = $derived($helpState.view);
  const hits = $derived(view.kind === 'search' ? searchChapters(view.query) : []);

  // Das Suchfeld folgt der Ansicht, auch bei „Zurück“ zu einer Suche.
  $effect(() => {
    query = view.kind === 'search' ? view.query : '';
  });

  // Esc im Suchfeld leert zuerst die Suche; erst ein leeres Feld schliesst die Hilfe.
  function onSearchKey(event: KeyboardEvent) {
    if (event.key !== 'Escape' || !query) return;
    event.preventDefault();
    event.stopPropagation();
    query = '';
    showToc();
  }

  function onSearch(event: Event) {
    query = (event.currentTarget as HTMLInputElement).value;
    if (query.trim()) showSearch(query);
    else showToc();
  }

  function open(slug: string) {
    query = '';
    openHelp(slug);
  }

  // Links im Kapitel: nur Kapitel-Links öffnen etwas; nichts verlässt das Programm.
  function onContentClick(event: MouseEvent) {
    const link = (event.target as Element).closest('a');
    if (!link) return;
    event.preventDefault();
    const slug = slugFromHref(link.getAttribute('href') ?? '');
    if (slug) open(slug);
  }

  function openPdf() {
    if (version) openWebsite(pdfUrl(version)).catch(() => undefined);
  }
</script>

<section class="help" role="complementary" aria-label="Hilfe">
  <header>
    <div class="row">
      {#if $helpState.history.length > 0}
        <button onclick={helpBack} title="Vorherige Hilfe-Seite">Zurück</button>
      {/if}
      <button onclick={() => { query = ''; showToc(); }} title="Inhaltsverzeichnis des Handbuchs">Inhalt</button>
      <span class="spacer"></span>
      <button class="close" onclick={closeHelp} title="Hilfe schliessen" aria-label="Hilfe schliessen">×</button>
    </div>
    <input type="search" placeholder="Im Handbuch suchen" value={query} oninput={onSearch} onkeydown={onSearchKey} />
  </header>

  <div class="content">
    {#if view.kind === 'chapter'}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions (Links darin sind per Tastatur bedienbar) -->
      <article onclick={onContentClick}>{@html renderChapter(view.slug)}</article>
    {:else if view.kind === 'search'}
      {#if hits.length === 0}
        <p class="muted">Keine Treffer. Versuchen Sie ein anderes Wort oder öffnen Sie das Inhaltsverzeichnis.</p>
      {:else}
        <ul class="hits">
          {#each hits as hit (hit.slug)}
            <li>
              <button class="hit" onclick={() => open(hit.slug)}>
                <strong>{hit.title}</strong>
                <span class="muted">{hit.snippet}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    {:else}
      <h1>Handbuch</h1>
      {#each PARTS as part (part.title)}
        <h2>{part.title}</h2>
        <ul class="toc">
          {#each part.chapters as chapter (chapter.slug)}
            <li><button class="link" onclick={() => open(chapter.slug)}>{chapter.title}</button></li>
          {/each}
        </ul>
      {/each}
    {/if}
  </div>

  <footer>
    <button onclick={openPdf} disabled={!version} title="Das ganze Handbuch zum Ausdrucken (braucht Internet)">
      Handbuch als PDF
    </button>
    <button onclick={() => reportOpen.set(true)} title="Bericht für den Entwickler erstellen">Problem melden</button>
  </footer>
</section>

<style>
  .help { display: flex; flex-direction: column; height: 100vh; border-left: 1px solid var(--border); background: var(--bg-muted); }
  header { padding: 0.75rem; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.5rem; }
  .row { display: flex; gap: 0.4rem; align-items: center; }
  .spacer { flex: 1; }
  .close { font-size: 1.1rem; line-height: 1; padding: 0.2rem 0.55rem; }
  input { width: 100%; }
  .content { flex: 1; overflow: auto; padding: 0.75rem 1rem; }
  .content :global(img) { max-width: 100%; border: 1px solid var(--border); border-radius: 4px; }
  .content :global(.bild-fehlt) { display: inline-block; padding: 0.3rem 0.5rem; border: 1px dashed var(--border); color: var(--text-muted); }
  .content :global(a[href]) { color: var(--accent); text-decoration: underline; cursor: pointer; }
  ul { list-style: none; padding: 0; margin: 0; }
  .toc li { margin: 0.15rem 0; }
  .hits li { margin: 0.4rem 0; }
  .hit { display: flex; flex-direction: column; align-items: flex-start; gap: 0.2rem; width: 100%; text-align: left; background: var(--bg); }
  footer { padding: 0.6rem 0.75rem; border-top: 1px solid var(--border); }
</style>
