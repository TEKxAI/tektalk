import { Component } from 'valdi_core/src/Component';

export class MePlugin extends Component {
  onRender() {
    <view padding={24} flex={1}>
      <label value="Me" font="title" />
      <label value="Profile, devices, privacy and application settings." />
    </view>;
  }
}
