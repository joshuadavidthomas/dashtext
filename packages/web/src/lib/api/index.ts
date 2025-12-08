import { Draft } from '@dashtext/lib';
import type { DraftData } from '@dashtext/lib';
import backend from './backend';

export type { DraftAPI, DraftData } from '@dashtext/lib';

export async function listDrafts(): Promise<Draft[]> {
  const data = await backend.list();
  return data.map((d) => new Draft(d));
}

export async function createDraft(): Promise<Draft> {
  const data = await backend.create();
  return new Draft(data);
}

export async function getDraft(id: number): Promise<Draft | null> {
  const data = await backend.get(id);
  return data ? new Draft(data) : null;
}

export async function saveDraft(id: number, content: string): Promise<DraftData> {
  return backend.save(id, content);
}

export async function deleteDraft(id: number): Promise<void> {
  return backend.delete(id);
}
