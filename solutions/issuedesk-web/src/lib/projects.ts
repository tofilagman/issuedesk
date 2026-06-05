import { api } from './api';
import type { Project } from './types';

/** Resolve a project by its key (routes are keyed by the human key, not the id). */
export async function getProjectByKey(key: string): Promise<Project> {
  const projects = await api.get<Project[]>('/api/projects');
  const found = projects.find((p) => p.key.toUpperCase() === key.toUpperCase());
  if (!found) throw new Error(`Project ${key} not found`);
  return found;
}
