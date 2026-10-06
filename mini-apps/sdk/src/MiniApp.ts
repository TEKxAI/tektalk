import { Capability, HostBridge } from '../../../plugins/sdk/src/Host';

export interface MiniAppContext {
  applicationId: string;
  version: string;
  grantedCapabilities: ReadonlySet<Capability>;
  host: HostBridge;
}

export abstract class MiniApp {
  constructor(protected readonly context: MiniAppContext) {}
  protected invoke<TRequest, TResponse>(capability: Capability, request: TRequest): Promise<TResponse> {
    if (!this.context.grantedCapabilities.has(capability)) throw new Error(`Capability denied: ${capability}`);
    return this.context.host.invoke<TRequest, TResponse>(capability, request);
  }
}
