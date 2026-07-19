import { writable, derived } from 'svelte/store';
import type { Writable, Readable } from 'svelte/store';
import type { User, Fighter, Technique, Video } from './lib/api/types';
import { getFighters } from './lib/api/fighters';
import { getTechniques } from './lib/api/techniques';
import { getVideos } from './lib/api/videos';
import { getMe } from './lib/api/auth';

export const token: Writable<string | null> = writable(
  typeof localStorage !== 'undefined' ? localStorage.getItem('ef_token') : null
);

export const currentUser: Writable<User | null> = writable(null);
export const techniques: Writable<Technique[]> = writable([]);
export const fighters: Writable<Fighter[]> = writable([]);
export const fighterVideoCounts: Writable<Record<string, number>> = writable({});
export const gallerySidebarOpen: Writable<boolean> = writable(false);

export const sortedFighters: Readable<Fighter[]> = derived(
  [fighters, fighterVideoCounts],
  ([$fighters, $counts]) => {
    return [...$fighters].sort((a, b) => {
      const countA = $counts[a.id] || 0;
      const countB = $counts[b.id] || 0;
      if (countB !== countA) return countB - countA;
      return a.display_name.localeCompare(b.display_name);
    });
  }
);

token.subscribe((value) => {
  if (typeof localStorage === 'undefined') return;
  if (value === null) {
    localStorage.removeItem('ef_token');
  } else {
    localStorage.setItem('ef_token', value);
  }
});

export async function updateFighterVideoCounts(videosList?: Video[]): Promise<void> {
  try {
    const list = videosList ?? await getVideos();
    const counts: Record<string, number> = {};
    for (const v of list) {
      if (v.fighter_a?.id) counts[v.fighter_a.id] = (counts[v.fighter_a.id] || 0) + 1;
      if (v.fighter_b?.id) counts[v.fighter_b.id] = (counts[v.fighter_b.id] || 0) + 1;
    }
    fighterVideoCounts.set(counts);
  } catch {
    // Ignore error if unauthenticated
  }
}

export async function initStores(): Promise<void> {
  const [fetchedFighters, fetchedTechniques, me] = await Promise.all([
    getFighters(),
    getTechniques(),
    getMe(),
  ]);
  fighters.set(fetchedFighters);
  techniques.set(fetchedTechniques);
  currentUser.set(me);
  void updateFighterVideoCounts();
}
