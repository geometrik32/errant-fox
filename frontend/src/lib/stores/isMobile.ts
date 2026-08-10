import { readable } from 'svelte/store';

export const isMobile = readable<boolean>(false, (set) => {
  if (typeof window === 'undefined') return;

  const mediaQuery = window.matchMedia('(max-width: 768px)');
  set(mediaQuery.matches);

  const handler = (e: MediaQueryListEvent) => {
    set(e.matches);
  };

  mediaQuery.addEventListener('change', handler);

  return () => {
    mediaQuery.removeEventListener('change', handler);
  };
});
