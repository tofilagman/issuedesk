export interface User {
  id: string;
  userName: string;
  email: string;
  displayName: string;
  role: number; // 0 member, 1 admin
  isActive: boolean;
  createdAt: string;
}

export interface Project {
  id: string;
  key: string;
  name: string;
  description?: string | null;
  issueSeq: number;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
}

export interface Member {
  userId: string;
  userName: string;
  displayName: string;
  email: string;
  role: number;
  addedAt: string;
}

export interface Label {
  id: string;
  projectId: string;
  name: string;
  color: string;
}

export interface IssueListItem {
  id: string;
  key: string;
  number: number;
  title: string;
  type: number;
  status: number;
  priority: number;
  assigneeId?: string | null;
  assigneeName?: string | null;
  createdAt: string;
  updatedAt: string;
  labels: Label[];
}

export interface IssueListResponse {
  items: IssueListItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface IssueDetail {
  id: string;
  projectId: string;
  projectKey: string;
  key: string;
  number: number;
  title: string;
  description?: string | null;
  type: number;
  status: number;
  priority: number;
  assigneeId?: string | null;
  assigneeName?: string | null;
  reporterId: string;
  reporterName: string;
  createdAt: string;
  updatedAt: string;
  labels: Label[];
}

export interface Comment {
  id: string;
  issueId: string;
  authorId: string;
  authorName: string;
  body: string;
  createdAt: string;
  updatedAt: string;
}

export interface Attachment {
  id: string;
  issueId: string;
  filename: string;
  storedPath: string;
  sizeBytes: number;
  mimeType: string;
  uploadedBy: string;
  createdAt: string;
}

export interface IssueLink {
  id: string; // link id
  linkType: number; // 0-4, relationship to the other issue (see LINK_TYPE_LABELS)
  issueId: string; // the other issue
  key: string; // e.g. "WAT-3"
  number: number;
  title: string;
  status: number;
  projectKey: string;
}

export interface Activity {
  id: string;
  issueId: string;
  actorId: string;
  actorName: string;
  action: number;
  field?: string | null;
  oldValue?: string | null;
  newValue?: string | null;
  createdAt: string;
}

// ---- enum labels ----
export const TYPE_LABELS = ['Bug', 'Task', 'Story', 'Epic'];
export const STATUS_LABELS = ['To Do', 'In Progress', 'In Review', 'Done'];
export const PRIORITY_LABELS = ['Low', 'Medium', 'High', 'Urgent'];

// Chart colors, indexed to match the *_LABELS arrays above.
export const STATUS_COLORS = ['#94a3b8', '#0ea5e9', '#f59e0b', '#10b981']; // todo, in-prog, review, done
export const PRIORITY_COLORS = ['#94a3b8', '#0ea5e9', '#f59e0b', '#f43f5e']; // low, med, high, urgent
export const TYPE_COLORS = ['#f43f5e', '#0ea5e9', '#10b981', '#8b5cf6']; // bug, task, story, epic

export interface AssigneeStat {
  userId: string;
  displayName: string;
  count: number;
}

export interface ProjectStats {
  total: number;
  open: number;
  done: number;
  unassigned: number;
  createdLast7: number;
  resolvedLast7: number;
  byStatus: number[];
  byType: number[];
  byPriority: number[];
  byAssignee: AssigneeStat[];
}

export interface ProjectSummary {
  key: string;
  name: string;
  total: number;
  done: number;
}

export interface SystemStats {
  projects: number;
  issues: number;
  users: number;
  open: number;
  done: number;
  createdLast7: number;
  byStatus: number[];
  byType: number[];
  byPriority: number[];
  topProjects: ProjectSummary[];
}

// Link relationship as seen from the current issue (matches backend facing type).
export const LINK_TYPE_LABELS = [
  'Relates to',
  'Blocks',
  'Is blocked by',
  'Duplicates',
  'Is duplicated by'
];
// Display order for grouping linked issues.
export const LINK_TYPE_ORDER = [1, 2, 0, 3, 4];

export const STATUS_COLUMNS = [
  { value: 0, label: 'To Do' },
  { value: 1, label: 'In Progress' },
  { value: 2, label: 'In Review' },
  { value: 3, label: 'Done' }
];

export const TYPE_META: Record<number, { label: string; icon: string; color: string }> = {
  0: { label: 'Bug', icon: '🐞', color: 'text-rose-600' },
  1: { label: 'Task', icon: '✓', color: 'text-sky-600' },
  2: { label: 'Story', icon: '📗', color: 'text-emerald-600' },
  3: { label: 'Epic', icon: '🏔', color: 'text-violet-600' }
};

export const PRIORITY_META: Record<number, { label: string; color: string }> = {
  0: { label: 'Low', color: 'bg-slate-200 text-slate-700' },
  1: { label: 'Medium', color: 'bg-sky-200 text-sky-800' },
  2: { label: 'High', color: 'bg-amber-200 text-amber-800' },
  3: { label: 'Urgent', color: 'bg-rose-200 text-rose-800' }
};
