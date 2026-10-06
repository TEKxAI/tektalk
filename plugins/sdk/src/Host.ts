export type Capability =
  | 'message.read' | 'message.send' | 'media.select' | 'media.upload' | 'call.start'
  | 'ai.chat' | 'ai.read_selected_context'
  | 'profile.read' | 'profile.write' | 'device.read' | 'consent.manage'
  | 'camera.capture' | 'location.read' | 'notifications.manage' | 'share.open';

export interface HostBridge {
  invoke<TRequest, TResponse>(capability: Capability, request: TRequest): Promise<TResponse>;
}

export interface ConversationSummary { id: string; title: string; preview: string; unreadCount: number; }
export interface ProfileSummary { displayName: string; identifier: string; activeDevices: number; }
