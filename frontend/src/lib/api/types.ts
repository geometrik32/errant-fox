export interface User {
  id: string;
  username: string;
  display_name: string;
  is_admin: boolean;
  avatar_url: string;
  color: string;
}

export interface Fighter {
  id: string;
  username: string;
  display_name: string;
  avatar_url: string;
  color: string;
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
  my_result: 'hit' | 'miss';
  opponent_technique_id: number | null;
  opponent_technique_name: string | null;
  opponent_hit_zone: string | null;
  opponent_result: 'hit' | 'miss';
}

export interface VideoFighter {
  id: string;
  display_name: string;
  avatar_url: string;
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
  result_a: 'hit' | 'miss' | 'blocked' | null;
  technique_b_id: number | null;
  hit_zone_b: string | null;
  result_b: 'hit' | 'miss' | 'blocked' | null;
}

export interface Comment {
  id: number;
  author: VideoFighter;
  timestamp_ms: number;
  text: string;
  reply_to_id: number | null;
  created_at: string;
}

export interface VideoFull {
  id: string;
  date: string;
  fighter_a: VideoFighter | null;
  fighter_b: VideoFighter | null;
  stream_url: string;
  duration_ms: number;
  bouts: Bout[];
  comments: Comment[];
}

export interface Technique {
  id: number;
  name: string;
}

export interface VideoShort {
  id: string;
  date: string;
  preview_url: string;
}
