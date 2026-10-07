import { Component } from 'valdi_core/src/Component';
import { fallbackUIEnvironment, pluginPalette, pluginText, UIEnvironment } from '../../sdk/src/Host';

export class AIPlugin extends Component {
  ui: UIEnvironment = fallbackUIEnvironment;
  setUIEnvironment(ui: UIEnvironment) { this.ui = ui; }
  onRender() {
    const palette = pluginPalette(this.ui);
    const text = { title: { en: 'AI', vi: 'AI' }, detail: { en: 'AI sessions are provided by the TEKtalk AI orchestrator.', vi: 'Phiên AI được cung cấp bởi bộ điều phối AI của TEKtalk.' } };
    <view padding={this.ui.layoutClass === 'expanded' ? 32 : 24} flex={1} backgroundColor={palette.background}>
      <label value={pluginText(this.ui, text, 'title')} font="title" color={palette.foreground} />
      <label value={pluginText(this.ui, text, 'detail')} color={palette.secondary} />
    </view>;
  }
}
