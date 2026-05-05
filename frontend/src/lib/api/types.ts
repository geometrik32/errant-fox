export interface User {
  id: string;
  username: string;
  display_name: string;
  is_admin: boolean;
  avatar_url: string;
  color: string | null;
}

export interface Fighter {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string;
  color: string | null;
  is_admin: boolean;
}

export function resolveColor(id: string, color: string | null | undefined): string {
  if (color) return color;
  let hash = 0;
  for (const ch of id) hash = (hash * 31 + ch.charCodeAt(0)) & 0x7fffffff;
  return `hsl(${hash % 360}, 55%, 48%)`;
}

export interface FighterBout {
  id: number;
  video_id: string;
  video_date: string;
  opponent_id: string;
  opponent_name: string;
  order_index: number;
  time_start_ms: number;
  time_end_ms: number;
  my_score: number;
  opponent_score: number;
  my_technique_id: number | null;
  my_technique_name: string | null;
  my_hit_zone: string | null;
  my_result: 'hit' | 'miss' | 'blocked' | 'late' | 'no_strike' | 'disqualification' | 'afterblow' | null;
  opponent_technique_id: number | null;
  opponent_technique_name: string | null;
  opponent_hit_zone: string | null;
  opponent_result: 'hit' | 'miss' | 'blocked' | 'late' | 'no_strike' | 'disqualification' | 'afterblow' | null;
}

export interface VideoFighter {
  id: string;
  display_name: string;
  avatar_url: string;
  color: string | null;
}

export interface Video {
  id: string;
  date: string;
  fighter_a: VideoFighter | null;
  fighter_b: VideoFighter | null;
  total_score_a?: number;
  total_score_b?: number;
  is_tagged: boolean;
  preview_url: string;
  preview_count: number;
}

export interface Bout {
  id: number;
  order_index: number;
  time_start_ms: number;
  time_end_ms: number;
  score_a: number;
  score_b: number;
  technique_a_id: number | null;
  hit_zone_a: string | null;
  result_a: 'hit' | 'miss' | 'blocked' | 'late' | 'no_strike' | 'disqualification' | 'afterblow' | null;
  technique_b_id: number | null;
  hit_zone_b: string | null;
  result_b: 'hit' | 'miss' | 'blocked' | 'late' | 'no_strike' | 'disqualification' | 'afterblow' | null;
}

export interface Comment {
  id: number;
  author: VideoFighter;
  timestamp_ms: number;
  text: string;
  reply_to_id: number | null;
  created_at: string;
  likes: number;
  dislikes: number;
  my_reaction: 'like' | 'dislike' | null;
  bout_id?: number | null;
}

export interface VideoFull {
  id: string;
  date: string;
  fighter_a: VideoFighter | null;
  fighter_b: VideoFighter | null;
  stream_url: string;
  duration_ms: number;
  fps?: number | null;
  bouts: Bout[];
  comments: Comment[];
}

export interface Technique {
  id: number;
  name: string;
  description?: string | null;
}

export interface VideoShort {
  id: string;
  date: string;
  preview_url: string;
}

// Computes human-readable video IDs: e.g. "СМ_26.02.01_01"
export function buildVideoLabels(bouts: FighterBout[], fighterName: string): Map<string, string> {
  const myInitial = (fighterName.trim()[0] ?? '?').toUpperCase();

  // First occurrence of each video_id → opponent name
  const videoOpponent = new Map<string, string>();
  const videoDates = new Map<string, string>();
  for (const b of bouts) {
    if (!videoOpponent.has(b.video_id)) {
      videoOpponent.set(b.video_id, b.opponent_name);
      videoDates.set(b.video_id, b.video_date);
    }
  }

  // Group videos by date, sorted by video_id within each date for stability
  const dateGroups = new Map<string, string[]>();
  for (const [vid, date] of videoDates) {
    if (!dateGroups.has(date)) dateGroups.set(date, []);
    dateGroups.get(date)!.push(vid);
  }
  for (const vids of dateGroups.values()) vids.sort();

  const result = new Map<string, string>();
  for (const [date, vids] of dateGroups) {
    const yy = date.slice(2, 4);
    const mm = date.slice(5, 7);
    const dd = date.slice(8, 10);
    const datePart = `${yy}.${mm}.${dd}`;
    vids.forEach((vid, idx) => {
      const oppInitial = ((videoOpponent.get(vid) ?? '').trim()[0] ?? '?').toUpperCase();
      result.set(vid, `${myInitial}${oppInitial}_${datePart}_${String(idx + 1).padStart(2, '0')}`);
    });
  }
  return result;
}

export interface SearchResult {
  comment_id: number;
  comment_text: string;
  author_id: string;
  author_name: string;
  timestamp_ms: number;
  video_id: string;
  video_date: string;
  fighter_a_name: string | null;
  fighter_b_name: string | null;
  bout_id: number | null;
  bout_order_index: number | null;
}
