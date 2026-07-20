export type UserRole = 'fighter' | 'guest' | 'retired';

export interface User {
  id: string;
  username: string;
  display_name: string;
  is_admin: boolean;
  avatar_url: string;
  color: string | null;
  vk_id?: string | null;
  role: UserRole;
}

export interface Fighter {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string;
  color: string | null;
  is_admin: boolean;
  vk_id?: string | null;
  role: UserRole;
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
  is_unmarked?: boolean;
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
  is_ai_labeled: boolean;
  is_analyzing?: boolean;
  is_queued?: boolean;
  has_transcript?: boolean;
  preview_url: string;
  preview_count: number;
  seafile_path?: string;
  seafile_web_url?: string;
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
  is_ai?: boolean;
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
  is_ai_labeled: boolean;
  is_analyzing?: boolean;
  is_queued?: boolean;
  has_transcript?: boolean;
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
