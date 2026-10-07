import { Component } from 'valdi_core/src/Component';
import { fallbackUIEnvironment, pluginPalette, pluginText, UIEnvironment } from '../../sdk/src/Host';

export class MessagePlugin extends Component {
  ui: UIEnvironment = fallbackUIEnvironment;
  setUIEnvironment(ui: UIEnvironment) { this.ui = ui; }
  onRender() {
    const palette = pluginPalette(this.ui);
    const text = { title: { en: 'Message', vi: 'Tin nhắn' }, detail: { en: 'Conversation list is provided by the TEKtalk message repository.', vi: 'Danh sách hội thoại được cung cấp bởi kho tin nhắn TEKtalk.' } };
    <view padding={this.ui.layoutClass === 'expanded' ? 32 : 24} flex={1} backgroundColor={palette.background}>
      <label value={pluginText(this.ui, text, 'title')} font="title" color={palette.foreground} />
      <label value={pluginText(this.ui, text, 'detail')} color={palette.secondary} />
    </view>;
  }
}
