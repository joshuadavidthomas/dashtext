import { DraftClient } from '@dashtext/lib/api';
import backend from './backend';

export const drafts = new DraftClient(backend);
