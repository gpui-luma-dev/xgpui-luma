// Framework-free screenshot galleries. Images remain readable without JavaScript.
(() => {
    const galleries = document.querySelectorAll('[data-image-carousel]');
    if (!galleries.length) return;
    const style = document.createElement('style');
    style.textContent = `
        [data-image-carousel]{margin:24px 0;min-width:0}
        [data-image-carousel] figure{margin:0}
        [data-image-carousel] img{display:block;width:100%;height:auto}
        [data-image-carousel][data-ready] .image-carousel-track{display:flex;overflow-x:auto;scroll-snap-type:x mandatory;overscroll-behavior-x:contain;scrollbar-width:thin}
        [data-image-carousel][data-ready] figure{flex:0 0 100%;min-width:0;scroll-snap-align:start;scroll-snap-stop:always}
        [data-image-carousel] .image-carousel-stage{position:relative}
        [data-image-carousel] .image-carousel-controls{position:absolute;top:50%;left:0;right:0;transform:translateY(-50%);display:flex;justify-content:space-between;padding:0 12px;pointer-events:none}
        [data-image-carousel] .image-carousel-controls button{pointer-events:auto;width:44px;height:44px;display:grid;place-items:center;border:1px solid var(--line);border-radius:50%;background:var(--bg);color:var(--fg);font-size:24px;line-height:1;padding:0;box-shadow:0 2px 8px #0003}
        [data-image-carousel] .image-carousel-controls button:hover:not(:disabled){background:var(--hover)}
        [data-image-carousel] .image-carousel-count{display:block;text-align:center;margin-top:8px;font-size:12px;color:var(--muted)}
        [data-image-carousel] .image-carousel-track:focus-visible{outline:2px solid var(--fg);outline-offset:4px}
    `;
    document.head.append(style);
    galleries.forEach(gallery => {
        const track = gallery.querySelector('.image-carousel-track');
        const slides = [...track.children];
        if (slides.length < 2) return;
        let current = 0;
        const controls = document.createElement('div');
        controls.className = 'image-carousel-controls';
        const previous = document.createElement('button');
        previous.type = 'button';
        previous.textContent = '‹';
        previous.setAttribute('aria-label', 'Previous screenshot');
        const next = document.createElement('button');
        next.type = 'button';
        next.textContent = '›';
        next.setAttribute('aria-label', 'Next screenshot');
        const count = document.createElement('span');
        count.className = 'image-carousel-count';
        count.setAttribute('aria-live', 'polite');
        count.setAttribute('aria-atomic', 'true');
        function update() {
            const index = Math.max(0, Math.min(slides.length - 1,
                Math.round(track.scrollLeft / track.clientWidth)));
            current = index;
            const text = `${index + 1} / ${slides.length}`;
            if (count.textContent !== text) count.textContent = text;
            previous.disabled = index === 0;
            next.disabled = index === slides.length - 1;
        }
        function go(index) {
            const target = Math.max(0, Math.min(slides.length - 1, index));
            track.scrollTo({ left: target * track.clientWidth,
                behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'instant' : 'smooth' });
        }
        previous.addEventListener('click', () => go(current - 1));
        next.addEventListener('click', () => go(current + 1));
        track.tabIndex = 0;
        track.setAttribute('aria-label', 'Screenshots; use left and right arrow keys to browse');
        gallery.addEventListener('keydown', event => {
            if (event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
            const targets = { ArrowLeft: current - 1, ArrowRight: current + 1, Home: 0, End: slides.length - 1 };
            if (!(event.key in targets)) return;
            event.preventDefault();
            go(targets[event.key]);
        });
        track.addEventListener('scroll', update, { passive: true });
        controls.append(previous, next);
        const stage = document.createElement('div');
        stage.className = 'image-carousel-stage';
        track.before(stage);
        stage.append(track, controls);
        gallery.append(count);
        gallery.dataset.ready = '';
        update();
        new ResizeObserver(() => {
            track.scrollTo({ left: current * track.clientWidth, behavior: 'instant' });
        }).observe(track);
    });
})();
