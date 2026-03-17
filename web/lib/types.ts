export type TaskStatus = "todo" | "doing" | "done";
export type Visibility = "public" | "unlisted" | "private";

export interface Commit {
  id: string;
  sha: string;
  message: string;
  committed_at: string;
  verified: boolean;
}

export interface Task {
  id: string;
  task_key: string;
  title: string;
  status: TaskStatus;
  done_at: string | null;
  commits: Commit[];
}

export interface Portfolio {
  id: string;
  slug: string;
  project_id: string;
  visibility: Visibility;
  published_at: string;
  tasks: Task[];
}
