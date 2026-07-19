import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';
import type { User, Fighter, Technique } from './lib/api/types';
import { getFighters } from './lib/api/fighters';
import { getTechniques } from './lib/api/techniques';
import { getMe } from './lib/api/auth';

export const token: Writable<string | null> = writable(
  typeof localStorage !== 'undefined' ? localStorage.getItem('ef_token') : null
);

export const currentUser: Writable<User | null> = writable(null);
export const techniques: Writable<Technique[]> = writable([]);
export const fighters: Writable<Fighter[]> = writable([]);
export const gallerySidebarOpen: Writable<boolean> = writable(false);

token.subscribe((value) => {
  if (typeof localStorage === 'undefined') return;
  if (value === null) {
    localStorage.removeItem('ef_token');
  } else {
    localStorage.setItem('ef_token', value);
  }
});

export async function initStores(): Promise<void> {
  const [fetchedFighters, fetchedTechniques, me] = await Promise.all([
    getFighters(),
    getTechniques(),
    getMe(),
  ]);
  fighters.set(fetchedFighters);
  techniques.set(fetchedTechniques);
  currentUser.set(me);
}
