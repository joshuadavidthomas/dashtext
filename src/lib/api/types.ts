export interface DraftData {
  id: number;
  content: string;
  created_at: string;
  modified_at: string;
}

export interface DraftAPI {
  list(): Promise<DraftData[]>;
  create(): Promise<DraftData>;
  get(id: number): Promise<DraftData | null>;
  save(id: number, content: string): Promise<DraftData>;
  delete(id: number): Promise<void>;
}
