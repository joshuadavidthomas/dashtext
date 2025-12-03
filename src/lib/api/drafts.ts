import { invoke } from '@tauri-apps/api/core';
import { Draft, type DraftData } from '$lib/stores/drafts.svelte';

export async function listDrafts(): Promise<Draft[]> {
	const data = await invoke<DraftData[]>('draft_list');
	return data.map((d) => new Draft(d));
}

export async function createDraft(): Promise<Draft> {
	const data = await invoke<DraftData>('draft_create');
	return new Draft(data);
}

export async function getDraft(id: number): Promise<Draft | null> {
	const data = await invoke<DraftData | null>('draft_get', { id });
	return data ? new Draft(data) : null;
}

export async function saveDraft(id: number, content: string): Promise<DraftData> {
	return invoke<DraftData>('draft_save', { id, content });
}

export async function deleteDraft(id: number): Promise<void> {
	return invoke('draft_delete', { id });
}
