import { derived, writable } from "svelte/store";

export type NotificationVariant = "success" | "info" | "error";

export interface NotificationAction {
  label: string;
  onClick: () => void;
}

export interface NotificationInput {
  title?: string;
  message: string;
  variant?: NotificationVariant;
  action?: NotificationAction;
}

export interface Notification extends NotificationInput {
  id: string;
  variant: NotificationVariant;
}

const MAX_VISIBLE = 3;

const notificationsStore = writable<Notification[]>([]);

let nextId = 1;

export const notifications = {
  subscribe: notificationsStore.subscribe,
};

export const visibleNotifications = derived(notificationsStore, (items) =>
  items.slice(0, MAX_VISIBLE),
);

export function pushNotification(input: NotificationInput): string {
  const notification: Notification = {
    id: `notification-${nextId++}`,
    title: input.title,
    variant: input.variant ?? "info",
    message: input.message,
    action: input.action,
  };

  notificationsStore.update((items) => [notification, ...items]);
  return notification.id;
}

export function dismissNotification(id: string): void {
  notificationsStore.update((items) => items.filter((item) => item.id !== id));
}

export function clearNotifications(): void {
  notificationsStore.set([]);
}
