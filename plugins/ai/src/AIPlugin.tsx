import { Component } from 'valdi_core/src/Component';

export class AIPlugin extends Component {
  onRender() {
    <view padding={24} flex={1}>
      <label value="AI" font="title" />
      <label value="AI sessions are provided by the TEKtalk AI orchestrator." />
    </view>;
  }
}
