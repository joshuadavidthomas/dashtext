import type { Draft } from '../stores';

export interface DraftData {
  uuid: string;
  content: string;
  created_at: string;
  modified_at: string;
  deleted_at?: string;
  archived?: boolean;
  pinned?: boolean;
}

// Internal type with both IDs for backend use
export interface DraftInternal extends DraftData {
  id: number;
}

export interface DraftAPI {
  list(): Promise<DraftData[]>;
  create(): Promise<DraftData>;
  get(uuid: string): Promise<DraftData | null>;
  save(uuid: string, content: string): Promise<DraftData>;
  archive(uuid: string): Promise<DraftData>;
  unarchive(uuid: string): Promise<DraftData>;
  pin(uuid: string): Promise<DraftData>;
  unpin(uuid: string): Promise<DraftData>;
  restore(uuid: string): Promise<DraftData>;
  delete(uuid: string): Promise<void>;
  hardDelete(uuid: string): Promise<void>;
}


