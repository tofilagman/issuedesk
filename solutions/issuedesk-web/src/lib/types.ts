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
