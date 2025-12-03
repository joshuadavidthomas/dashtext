/**
 * Raw draft data shape from Tauri API
 */
export type DraftData = {
	id: number;
	content: string;
	created_at: string;
	modified_at: string;
};

/**
 * Draft - reactive draft model with derived presentation properties
 */
export class Draft {
	id: number;
	content = $state('');
	created_at: string;
	modified_at = $state('');

	title = $derived(this.content.split('\n')[0].trim() || 'Untitled');

	previewLines = $derived.by(() => {
		const lines = this.content.split('\n').slice(1);
		return lines.filter((line) => line.trim()).slice(0, 3);
	});

	formattedModifiedAt = $derived.by(() => {
		const value = this.modified_at;
		const asNumber = parseInt(value);
		const date =
			!isNaN(asNumber) && value === String(asNumber)
				? new Date(asNumber * 1000) // Unix timestamp (seconds)
				: new Date(value); // ISO/RFC 3339 string

		if (isNaN(date.getTime())) return 'Unknown date';

		return date.toLocaleString(undefined, {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	});

	constructor(data: DraftData) {
		this.id = data.id;
		this.content = data.content;
		this.created_at = data.created_at;
		this.modified_at = data.modified_at;
	}
}
