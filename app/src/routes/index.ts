export const ROUTES = {
  INDEX: "/",
  INVITE: "/invite",
  PREVIEW: "preview",
  join: (invitePackage: string = ":package") =>
    `/join/${invitePackage}` as const,
  chat: (groupId: string = ":groupId") => `/chat/${groupId}` as const,
  DEBUG: "/debug",
} as const;
