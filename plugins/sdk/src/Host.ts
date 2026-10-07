export type Capability =
  | 'message.read' | 'message.send' | 'media.select' | 'media.upload' | 'call.start'
  | 'ai.chat' | 'ai.read_selected_context'
  | 'profile.read' | 'profile.write' | 'device.read' | 'consent.manage'
  | 'camera.capture' | 'location.read' | 'notifications.manage' | 'share.open'
  | 'ui.context';

export type ColorScheme = 'light' | 'dark';
export type LayoutClass = 'compact' | 'expanded';
export interface UIEnvironment {
  locale: string;
  colorScheme: ColorScheme;
  layoutClass: LayoutClass;
  contentSize: 'normal' | 'large' | 'accessibility';
}

export const fallbackUIEnvironment: UIEnvironment = {
  locale: 'en', colorScheme: 'light', layoutClass: 'compact', contentSize: 'normal'
};

export function pluginText(environment: UIEnvironment, values: Record<string, { en: string; vi: string }>, key: string): string {
  const language = environment.locale.toLowerCase().startsWith('vi') ? 'vi' : 'en';
  return values[key]?.[language] ?? key;
}

export function pluginPalette(environment: UIEnvironment) {
  return environment.colorScheme === 'dark'
    ? { background: '#101318', foreground: '#F2F4F8', secondary: '#B8C0CC' }
    : { background: '#FFFFFF', foreground: '#171A1F', secondary: '#566170' };
}

export interface HostBridge {
  invoke<TRequest, TResponse>(capability: Capability, request: TRequest): Promise<TResponse>;
}

export interface ConversationSummary { id: string; title: string; preview: string; unreadCount: number; }
export interface ProfileSummary { displayName: string; identifier: string; activeDevices: number; }
