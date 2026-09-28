import { Component } from 'valdi_core/src/Component';

export class MessagePlugin extends Component {
  onRender() {
    <view padding={24} flex={1}>
      <label value="Message" font="title" />
      <label value="Conversation list is provided by the TEKtalk message repository." />
    </view>;
  }
}
