import { Draft } from '../stores';
import type { DraftAPI, DraftData } from './types';

/**
 * Client for draft operations - wraps a backend implementation
 * and returns Draft class instances with reactive properties.
 */
export class DraftClient {
  constructor(private backend: DraftAPI) {}

  async list(): Promise<Draft[]> {
    const data = await this.backend.list();
    return data.map((d) => new Draft(d));
  }

  async create(): Promise<Draft> {
    const data = await this.backend.create();
    return new Draft(data);
  }

  async get(id: number): Promise<Draft | null> {
    const data = await this.backend.get(id);
    return data ? new Draft(data) : null;
  }

  async save(id: number, content: string): Promise<DraftData> {
    return this.backend.save(id, content);
  }

  async delete(id: number): Promise<void> {
    return this.backend.delete(id);
  }
}
