<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { openWebsite } from '../lib/desktop';
  import { reportOpen } from '../lib/diagnostics';
  import { PARTS, pdfUrl, renderChapter, searchChapters, slugFromHref } from '../lib/handbook';
  import { closeHelp, helpBack, helpState, openHelp, showSearch, showToc } from '../lib/help';
  import { HELP_WIDTH, helpWidth, isWide, setHelpWidth, toggleWide } from '../lib/helpWidth';
  import { helpJoke } from '../lib/osternest/sayings';

  let version = $state('');
  let query = $state('');
  let zoomed = $state<{ src: string; alt: string } | null>(null);
  let stopDrag: (() => void) | null = null;

  onMount(async () => {
    try {
      version = await (await import('@tauri-apps/api/app')).getVersion();
    } catch {
      version = '';
    }
  });

  const view = $derived($helpState.view);
  const hits = $derived(view.kind === 'search' ? searchChapters(view.query) : []);
  const joke = $derived(view.kind === 'search' ? helpJoke(view.query) : null);

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

  // Bilder im Kapitel lassen sich anklicken und erscheinen dann gross über dem ganzen Fenster.
  function markImages(article: HTMLElement, _html: string) {
    const mark = () => {
      for (const img of article.querySelectorAll('img[src]')) {
        img.setAttribute('role', 'button');
        img.setAttribute('tabindex', '0');
        img.setAttribute('aria-label', `Bild vergrössern: ${img.getAttribute('alt') ?? ''}`.trim());
        img.setAttribute('title', 'Klicken zum Vergrössern');
      }
    };
    mark();
    return { update: mark };
  }

  function zoomImage(img: HTMLImageElement) {
    zoomed = { src: img.src, alt: img.alt };
  }

  function onContentKey(event: KeyboardEvent) {
    const target = event.target as Element;
    if (target instanceof HTMLImageElement && (event.key === 'Enter' || event.key === ' ')) {
      event.preventDefault();
      zoomImage(target);
    }
  }

  // Esc schliesst zuerst das grosse Bild; die Hilfe bleibt offen (offener Dialog, siehe handleHelpKey).
  function onWindowKey(event: KeyboardEvent) {
    if (zoomed && event.key === 'Escape') {
      event.preventDefault();
      zoomed = null;
    }
  }

  // Ziehen am linken Rand: Breite = Abstand des Zeigers vom rechten Fensterrand.
  function startDrag(event: PointerEvent) {
    event.preventDefault();
    const move = (e: PointerEvent) => setHelpWidth(window.innerWidth - e.clientX, window.innerWidth);
    const up = () => stopDrag?.();
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
    document.body.classList.add('resizing');
    stopDrag = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
      document.body.classList.remove('resizing');
      stopDrag = null;
    };
  }

  // Pfeiltasten am Rand: links = breiter, rechts = schmaler.
  function onHandleKey(event: KeyboardEvent) {
    const step = event.key === 'ArrowLeft' ? 40 : event.key === 'ArrowRight' ? -40 : 0;
    if (!step) return;
    event.preventDefault();
    setHelpWidth($helpWidth + step, window.innerWidth);
  }

  onDestroy(() => stopDrag?.());

  // Links im Kapitel: nur Kapitel-Links öffnen etwas; nichts verlässt das Programm.
  function onContentClick(event: MouseEvent) {
    if (event.target instanceof HTMLImageElement) {
      zoomImage(event.target);
      return;
    }
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

<svelte:window onkeydown={onWindowKey} />

<section class="help" role="complementary" aria-label="Hilfe">
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions (Trenner ist per Pfeiltasten bedienbar) -->
  <div
    class="handle"
    role="separator"
    aria-orientation="vertical"
    aria-label="Breite der Hilfe ändern"
    aria-valuenow={$helpWidth}
    tabindex="0"
    title="Ziehen, um die Hilfe breiter oder schmaler zu machen (Doppelklick: normale Breite)"
    onpointerdown={startDrag}
    ondblclick={() => setHelpWidth(HELP_WIDTH.normal, window.innerWidth)}
    onkeydown={onHandleKey}
  ></div>
  <header>
    <div class="row">
      {#if $helpState.history.length > 0}
        <button onclick={helpBack} title="Vorherige Hilfe-Seite">Zurück</button>
      {/if}
      <button onclick={() => { query = ''; showToc(); }} title="Inhaltsverzeichnis des Handbuchs">Inhalt</button>
      <span class="spacer"></span>
      <button onclick={() => toggleWide(window.innerWidth)} title="Hilfe breiter oder schmaler anzeigen">
        {isWide($helpWidth) ? 'Schmaler' : 'Breiter'}
      </button>
      <button class="close" onclick={closeHelp} title="Hilfe schliessen" aria-label="Hilfe schliessen">×</button>
    </div>
    <input type="search" placeholder="Im Handbuch suchen" value={query} oninput={onSearch} onkeydown={onSearchKey} />
  </header>

  <div class="content">
    {#if view.kind === 'chapter'}
      {@const html = renderChapter(view.slug)}
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions (Links und Bilder darin sind per Tastatur bedienbar) -->
      <article onclick={onContentClick} onkeydown={onContentKey} use:markImages={html}>{@html html}</article>
    {:else if view.kind === 'search'}
      {#if joke}<p class="osternest muted">{joke}</p>{/if}
      {#if hits.length === 0 && !joke}
        <p class="muted">Keine Treffer. Versuchen Sie ein anderes Wort oder öffnen Sie das Inhaltsverzeichnis.</p>
      {:else if hits.length > 0}
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

{#if zoomed}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (Esc und „Bild schliessen“ gehen per Tastatur) -->
  <div class="lightbox" onclick={() => (zoomed = null)}>
    <dialog open aria-label={zoomed.alt}>
      <img src={zoomed.src} alt={zoomed.alt} />
      <p class="caption">{zoomed.alt} <span class="muted">· Klicken oder Esc zum Schliessen</span></p>
    </dialog>
    <button class="lightbox-close" onclick={() => (zoomed = null)} aria-label="Bild schliessen" title="Bild schliessen (Esc)">×</button>
  </div>
{/if}

<style>
  .help { position: relative; display: flex; flex-direction: column; height: 100vh; min-width: 0; border-left: 1px solid var(--border); background: var(--bg-muted); }
  .handle { position: absolute; left: -4px; top: 0; bottom: 0; width: 8px; cursor: col-resize; z-index: 5; }
  .handle:hover, .handle:focus-visible { background: var(--accent); opacity: 0.35; outline: none; }
  :global(body.resizing) { cursor: col-resize; user-select: none; }
  header { padding: 0.75rem; border-bottom: 1px solid var(--border); display: flex; flex-direction: column; gap: 0.5rem; }
  .row { display: flex; gap: 0.4rem; align-items: center; }
  .spacer { flex: 1; }
  .close { font-size: 1.1rem; line-height: 1; padding: 0.2rem 0.55rem; }
  input { width: 100%; }
  .content { flex: 1; overflow: auto; padding: 0.75rem 1rem; }
  .content :global(img) { max-width: 100%; border: 1px solid var(--border); border-radius: 4px; cursor: zoom-in; }
  .content :global(img:focus-visible) { outline: 2px solid var(--accent); outline-offset: 2px; }
  .lightbox { position: fixed; inset: 0; z-index: 60; background: rgb(0 0 0 / 0.7); display: grid; place-items: center; cursor: zoom-out; padding: 1.5rem; }
  .lightbox dialog { position: static; margin: 0; padding: 0; border: none; background: transparent; max-width: 100%; max-height: 100%; display: flex; flex-direction: column; align-items: center; gap: 0.5rem; }
  .lightbox img { max-width: calc(100vw - 3rem); max-height: calc(100vh - 5rem); object-fit: contain; background: var(--bg); border-radius: 4px; box-shadow: 0 4px 24px rgb(0 0 0 / 0.4); }
  .caption { margin: 0; color: #fff; }
  .caption .muted { color: rgb(255 255 255 / 0.75); }
  .lightbox-close { position: fixed; top: 0.75rem; right: 0.75rem; font-size: 1.4rem; line-height: 1; padding: 0.25rem 0.65rem; }
  .content :global(.bild-fehlt) { display: inline-block; padding: 0.3rem 0.5rem; border: 1px dashed var(--border); color: var(--text-muted); }
  .content :global(a[href]) { color: var(--accent); text-decoration: underline; cursor: pointer; }
  ul { list-style: none; padding: 0; margin: 0; }
  .toc li { margin: 0.15rem 0; }
  .hits li { margin: 0.4rem 0; }
  .hit { display: flex; flex-direction: column; align-items: flex-start; gap: 0.2rem; width: 100%; text-align: left; background: var(--bg); }
  footer { padding: 0.6rem 0.75rem; border-top: 1px solid var(--border); }
</style>
