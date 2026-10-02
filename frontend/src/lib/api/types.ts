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
  if (id === 'guest') return '#475569';
  let hash = 0;
  for (const ch of id) hash = (hash * 31 + ch.charCodeAt(0)) & 0x7fffffff;
  return `hsl(${hash % 360}, 55%, 48%)`;
}

export function getContrastColor(color: string | null | undefined): string {
  if (!color) return '#ffffff';
  let c = color.trim();
  if (c.startsWith('hsl')) {
    const m = c.match(/hsl\(\s*\d+\s*,\s*\d+%\s*,\s*(\d+)%\s*\)/);
    if (m && parseInt(m[1], 10) > 65) return '#0f172a';
    return '#ffffff';
  }
  if (c.startsWith('#')) c = c.slice(1);
  if (c.length === 3) {
    c = c.split('').map(ch => ch + ch).join('');
  }
  if (c.length === 6) {
    const r = parseInt(c.slice(0, 2), 16) || 0;
    const g = parseInt(c.slice(2, 4), 16) || 0;
    const b = parseInt(c.slice(4, 6), 16) || 0;
    const yiq = (r * 299 + g * 587 + b * 114) / 1000;
    return yiq >= 160 ? '#0f172a' : '#ffffff';
  }
  return '#ffffff';
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
  is_ai?: boolean;
  is_tournament?: boolean;
  tournament_name?: string | null;
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
  is_optimized?: boolean;
  is_optimizing?: boolean;
  is_eligible_for_optimization?: boolean;
  has_transcript?: boolean;
  has_human_bouts?: boolean;
  has_h264?: boolean;
  is_transcoding?: boolean;
  is_tournament: boolean;
  tournament_name?: string | null;
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
  drawing?: string | null;
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
  is_optimized?: boolean;
  is_optimizing?: boolean;
  is_eligible_for_optimization?: boolean;
  has_transcript?: boolean;
  has_human_bouts?: boolean;
  has_h264?: boolean;
  is_transcoding?: boolean;
  is_tournament: boolean;
  tournament_name?: string | null;
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

export interface OptimizationCandidate {
  id: string;
  date: string;
  seafile_path: string;
  duration_ms: number | null;
  bouts_count: number;
  fighter_a_name: string | null;
  fighter_b_name: string | null;
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
}

export interface NotificationUserSummary {
  id: string;
  display_name: string;
  avatar_url: string | null;
  color: string | null;
}

export interface NotificationItem {
  id: number;
  user_id: string;
  actor: NotificationUserSummary;
  event_type: 'video_tag' | 'video_labeled' | 'bout_added' | 'bout_updated' | 'comment_reply' | 'video_comment' | string;
  video_id: string;
  video_title: string;
  bout_id: number | null;
  comment_id: number | null;
  payload: any;
  is_read: boolean;
  created_at: string;
}

export interface NotificationsResponse {
  items: NotificationItem[];
  total: number;
  unread_count: number;
}
