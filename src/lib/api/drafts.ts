import { invoke } from '@tauri-apps/api/core';

export type Draft = {
	id: number;
	content: string;
	created_at: string;
	modified_at: string;
};

export async function listDrafts(): Promise<Draft[]> {
	return invoke('draft_list');
}

export async function createDraft(): Promise<Draft> {
	return invoke('draft_create');
}

export async function getDraft(id: number): Promise<Draft | null> {
	return invoke('draft_get', { id });
}

export async function saveDraft(id: number, content: string): Promise<Draft> {
	return invoke('draft_save', { id, content });
}

export async function deleteDraft(id: number): Promise<void> {
	return invoke('draft_delete', { id });
}
