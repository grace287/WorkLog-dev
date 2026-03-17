import { z } from "zod";

export const CommitSchema = z.object({
  id: z.string(),
  sha: z.string(),
  message: z.string(),
  committed_at: z.string(),
  verified: z.boolean(),
});

export const TaskSchema = z.object({
  id: z.string(),
  task_key: z.string(),
  title: z.string(),
  status: z.enum(["todo", "doing", "done"]),
  done_at: z.string().nullable(),
  commits: z.array(CommitSchema).default([]),
});

export const PortfolioSchema = z.object({
  id: z.string(),
  slug: z.string(),
  project_id: z.string(),
  visibility: z.enum(["public", "unlisted", "private"]),
  published_at: z.string(),
  tasks: z.array(TaskSchema).default([]),
});

export type PortfolioData = z.infer<typeof PortfolioSchema>;
