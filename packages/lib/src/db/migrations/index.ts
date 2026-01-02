// Migration files bundled for web via Vite
// Desktop uses these same files via Rust include_str!

// @ts-ignore - Vite handles .sql?raw imports at build time
import migration0000 from './0000_clean_blockbuster.sql?raw';
// @ts-ignore - Vite handles .sql?raw imports at build time
import migration0001 from './0001_gigantic_sharon_ventura.sql?raw';
// @ts-ignore - Vite handles .sql?raw imports at build time
import migration0002 from './0002_add_settings_table.sql?raw';
// @ts-ignore - Vite handles .sql?raw imports at build time
import migration0003 from './0003_add_automerge_tables.sql?raw';
import journal from './meta/_journal.json';

export interface Migration {
	sql: string[];
	folderMillis: number;
	hash: string;
}

export const migrations: Migration[] = journal.entries.map((entry: any, i: number) => {
	const migrationFiles = [migration0000, migration0001, migration0002, migration0003];
	const sql = migrationFiles[i]
		.split('--> statement-breakpoint')
		.map((s: string) => s.trim())
		.filter(Boolean);

	return {
		sql,
		folderMillis: entry.when,
		hash: entry.tag,
	};
});

export { journal };
