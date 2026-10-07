import { Component } from 'valdi_core/src/Component';
import { fallbackUIEnvironment, pluginPalette, pluginText, UIEnvironment } from '../../sdk/src/Host';

export class MePlugin extends Component {
  ui: UIEnvironment = fallbackUIEnvironment;
  setUIEnvironment(ui: UIEnvironment) { this.ui = ui; }
  onRender() {
    const palette = pluginPalette(this.ui);
    const text = { title: { en: 'Me', vi: 'Tôi' }, detail: { en: 'Profile, devices, privacy and application settings.', vi: 'Hồ sơ, thiết bị, quyền riêng tư và cài đặt ứng dụng.' } };
    <view padding={this.ui.layoutClass === 'expanded' ? 32 : 24} flex={1} backgroundColor={palette.background}>
      <label value={pluginText(this.ui, text, 'title')} font="title" color={palette.foreground} />
      <label value={pluginText(this.ui, text, 'detail')} color={palette.secondary} />
    </view>;
  }
}
