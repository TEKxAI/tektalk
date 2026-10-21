#include "pch.h"
#include "MainWindow.xaml.h"
#include <winrt/Windows.UI.Xaml.Interop.h>
#include "Domain/ClientLogic.h"
#include "DesignMetrics.h"
#include <algorithm>
#if __has_include("MainWindow.g.cpp")
#include "MainWindow.g.cpp"
#endif
using namespace winrt;using namespace Windows::Foundation;using namespace Microsoft::UI::Xaml;using namespace Microsoft::UI::Xaml::Controls;
namespace winrt::TEKtalk::implementation {
namespace { void Aid(DependencyObject const& e,wchar_t const* id){Microsoft::UI::Xaml::Automation::AutomationProperties::SetAutomationId(e,id);} hstring Tr(wchar_t const* key){try{auto value=Microsoft::Windows::ApplicationModel::Resources::ResourceLoader().GetString(key);return value.empty()?hstring(key):value;}catch(hresult_error const&){return hstring(key);}} }
TextBlock MainWindow::Text(hstring const& value,double size,bool bold){TextBlock t;t.Text(value);t.FontSize(size);if(bold)t.FontWeight(Windows::UI::Text::FontWeights::SemiBold());return t;}
MainWindow::MainWindow(){InitializeComponent();root_=Root();try{ExtendsContentIntoTitleBar(true);AppWindow().Resize({1100,720});}catch(hresult_error const&){}ShowAuth();}
void MainWindow::ShowAuth(){root_.Children().Clear();auto shell=Grid();shell.ColumnDefinitions().Append(ColumnDefinition());shell.ColumnDefinitions().Append(ColumnDefinition());shell.ColumnDefinitions().GetAt(0).Width(GridLengthHelper::FromPixels(410));auto hero=StackPanel();hero.Background(Application::Current().Resources().Lookup(box_value(L"TekPrimaryBrush")).as<Media::Brush>());hero.Padding({48});hero.Spacing(18);auto brand=Text(L"TEKtalk",48,true);brand.Foreground(Application::Current().Resources().Lookup(box_value(L"TekOnPrimaryBrush")).as<Media::Brush>());hero.Children().Append(brand);auto subtitle=Text(Tr(L"AuthHero"),20);subtitle.Foreground(Application::Current().Resources().Lookup(box_value(L"TekOnPrimaryBrush")).as<Media::Brush>());hero.Children().Append(subtitle);auto core=Text(core_.Available()?Tr(L"CoreConnected"):Tr(L"CoreFallback"),13,true);core.Foreground(Application::Current().Resources().Lookup(box_value(L"TekOnPrimaryBrush")).as<Media::Brush>());hero.Children().Append(core);Grid::SetColumn(hero,0);shell.Children().Append(hero);auto form=StackPanel();form.Padding({64});form.Spacing(12);form.MaxWidth(520);form.HorizontalAlignment(HorizontalAlignment::Stretch);form.Children().Append(Text(register_mode_?Tr(L"CreateAccount"):challenge_.empty()?Tr(L"SignIn"):Tr(L"VerifyDevice"),34,true));if(challenge_.empty()){phone_=TextBox();Aid(phone_,L"auth.phone");phone_.Header(box_value(Tr(L"PhoneE164")));phone_.Text(L"+84");form.Children().Append(phone_);if(register_mode_){name_=TextBox();Aid(name_,L"auth.name");name_.Header(box_value(Tr(L"DisplayName")));form.Children().Append(name_);}password_=PasswordBox();Aid(password_,L"auth.password");password_.Header(box_value(Tr(L"Password")));form.Children().Append(password_);if(register_mode_){question_=TextBox();Aid(question_,L"auth.question");question_.Header(box_value(Tr(L"SecurityQuestion")));question_.Text(Tr(L"DefaultSecurityQuestion"));form.Children().Append(question_);}}answer_=TextBox();Aid(answer_,L"auth.answer");answer_.Header(box_value(challenge_.empty()?Tr(L"SecurityAnswer"):Tr(L"AnswerChallenge")));if(register_mode_||!challenge_.empty())form.Children().Append(answer_);status_=Text(L"");form.Children().Append(status_);auto submit=Button();Aid(submit,L"auth.submit");submit.Content(box_value(challenge_.empty()?(register_mode_?Tr(L"Register"):Tr(L"SignIn")):Tr(L"VerifyDevice")));submit.HorizontalAlignment(HorizontalAlignment::Stretch);Primary(submit);submit.Click([this](auto&&,auto&&){SubmitAuth();});form.Children().Append(submit);if(challenge_.empty()){auto toggle=Button();Aid(toggle,L"auth.toggle");toggle.Content(box_value(register_mode_?Tr(L"SwitchToLogin"):Tr(L"SwitchToRegister")));toggle.Click([this](auto&&,auto&&){register_mode_=!register_mode_;ShowAuth();});form.Children().Append(toggle);}Grid::SetColumn(form,1);shell.Children().Append(form);root_.Children().Append(shell);}
IAsyncAction MainWindow::SubmitAuth(){try{if(!challenge_.empty()){tokens_=tektalk::ApiClient::ParseTokens(co_await api_.Verify(challenge_,answer_.Text()));ShowHost();co_return;}if(!tektalk::ValidE164(phone_.Text().c_str())||!tektalk::ValidPassword(password_.Password().c_str())){status_.Text(Tr(L"InvalidCredentials"));co_return;}if(register_mode_){tokens_=tektalk::ApiClient::ParseTokens(co_await api_.Register(phone_.Text(),name_.Text(),password_.Password(),question_.Text(),answer_.Text()));ShowHost();}else{auto result=co_await api_.Login(phone_.Text(),password_.Password());if(result.HasKey(L"tokens")&&result.GetNamedValue(L"tokens").ValueType()==Windows::Data::Json::JsonValueType::Object){tokens_=tektalk::ApiClient::ParseTokens(result.GetNamedObject(L"tokens"));ShowHost();}else{challenge_=result.GetNamedString(L"challenge_id",L"");ShowAuth();status_.Text(result.GetNamedString(L"question",Tr(L"VerifyDevice")));}}}catch(hresult_error const&e){status_.Text(e.message());}}
void MainWindow::ShowHost() {
    root_.Children().Clear(); tabs_.clear(); recent_.clear(); messageContent_=nullptr;
    auto shell=Grid();
    ColumnDefinition left; left.Width(GridLengthHelper::FromPixels(tektalk::design::SidebarWidth));
    shell.ColumnDefinitions().Append(left); shell.ColumnDefinitions().Append(ColumnDefinition());
    auto sidebar=Grid(); sidebar.Background(Brush(L"TekSurfaceBrush"));
    RowDefinition body; body.Height(GridLengthHelper::FromValueAndType(1,GridUnitType::Star));
    RowDefinition bottom; bottom.Height(GridLengthHelper::Auto());
    sidebar.RowDefinitions().Append(body); sidebar.RowDefinitions().Append(bottom);
    sidebarContent_=StackPanel(); sidebarContent_.Padding({16,24,16,16}); sidebarContent_.Spacing(16);
    auto scroll=ScrollViewer(); scroll.Content(sidebarContent_); sidebar.Children().Append(scroll);
    auto tabRow=Grid(); tabRow.Padding({8,8,8,8}); tabRow.BorderThickness({0,1,0,0}); tabRow.BorderBrush(Brush(L"TekOutlineBrush"));
    Aid(tabRow,L"navigation.bottom");
    const std::tuple<hstring,hstring,hstring> entries[]={{Tr(L"Message"),L"\uE8F2",L"tab.Message"},{L"AI",L"\uE945",L"tab.AI"},{Tr(L"Me"),L"\uE77B",L"tab.Me"}};
    uint32_t index=0;
    for(auto const& entry:entries) {
        tabRow.ColumnDefinitions().Append(ColumnDefinition());
        Button button; button.HorizontalAlignment(HorizontalAlignment::Stretch);
        button.MinHeight(tektalk::design::TouchTarget); button.Padding({2,8,2,8}); button.BorderThickness({0}); button.CornerRadius({12,12,12,12});
        Aid(button,std::get<2>(entry).c_str());
        auto label=StackPanel(); label.Spacing(4); label.HorizontalAlignment(HorizontalAlignment::Center);
        auto icon=FontIcon(); icon.Glyph(std::get<1>(entry)); label.Children().Append(icon);
        auto caption=Text(std::get<0>(entry),11,true); caption.HorizontalAlignment(HorizontalAlignment::Center); label.Children().Append(caption);
        button.Content(label); auto selected=index++;
        button.Click([this,selected](auto&&,auto&&){if(selected==0)ShowMessage();else if(selected==1)ShowAI();else ShowMe();});
        Grid::SetColumn(button,selected); tabs_.push_back(button); tabRow.Children().Append(button);
    }
    Grid::SetRow(tabRow,1); sidebar.Children().Append(tabRow); shell.Children().Append(sidebar);
    content_=Grid(); content_.Background(Brush(L"TekBackgroundBrush")); Grid::SetColumn(content_,1); shell.Children().Append(content_);
    root_.Children().Append(shell); ShowMessage();
}
Media::Brush MainWindow::Brush(wchar_t const* key) { return Application::Current().Resources().Lookup(box_value(key)).as<Media::Brush>(); }
void MainWindow::Primary(Button const& button) {
    button.Style(Application::Current().Resources().Lookup(box_value(L"AccentButtonStyle")).as<Style>());
    button.Background(Brush(L"TekPrimaryBrush")); button.Foreground(Brush(L"TekOnPrimaryBrush"));
    for(auto key:{L"ButtonBackgroundPointerOver",L"ButtonBackgroundPressed"})button.Resources().Insert(box_value(key),Brush(L"TekPrimaryBrush"));
    for(auto key:{L"ButtonForegroundPointerOver",L"ButtonForegroundPressed"})button.Resources().Insert(box_value(key),Brush(L"TekOnPrimaryBrush"));
    button.MinHeight(tektalk::design::TouchTarget); button.CornerRadius({12,12,12,12});
}
void MainWindow::ShowSidebar(uint32_t index) {
    sidebarContent_.Children().Clear(); sidebarContent_.Children().Append(Text(L"TEKtalk",24,true));
    if(index==0) {
        sidebarContent_.Children().Append(Text(Tr(L"Message"),22,true));
        if(recent_.empty()) { auto hint=Text(Tr(L"MessageStartHint")); hint.TextWrapping(TextWrapping::Wrap); hint.Foreground(Brush(L"TekMutedBrush")); sidebarContent_.Children().Append(hint); }
        for(auto const& entry:recent_) {
            Button row; row.HorizontalAlignment(HorizontalAlignment::Stretch); row.HorizontalContentAlignment(HorizontalAlignment::Stretch);
            row.Padding({12,12,12,12}); row.CornerRadius({12,12,12,12}); row.BorderThickness({0}); row.Background(Brush(L"TekPrimaryContainerBrush"));
            Aid(row,(L"history."+entry.id).c_str());
            auto label=StackPanel(); label.Spacing(4); label.Children().Append(Text(Tr(L"DirectConversation"),14,true));
            auto preview=Text(entry.preview.empty()?entry.id:entry.preview,12); preview.Foreground(Brush(L"TekMutedBrush")); preview.TextWrapping(TextWrapping::Wrap); label.Children().Append(preview);
            row.Content(label);
            row.Click([this,entry](auto&&,auto&&){ShowMessage();conversation_.Text(entry.id);recipient_.Text(entry.recipient);LoadMessages();});
            sidebarContent_.Children().Append(row);
        }
    } else if(index==1) {
        sidebarContent_.Children().Append(Text(L"AI",22,true));
        auto description=Text(Tr(L"AiDescription")); description.TextWrapping(TextWrapping::Wrap); sidebarContent_.Children().Append(description);
    } else {
        sidebarContent_.Children().Append(Text(Tr(L"Me"),22,true));
        if(tokens_) { auto user=Text(tokens_->user_id,12); user.TextWrapping(TextWrapping::Wrap); sidebarContent_.Children().Append(user); }
    }
}
void MainWindow::RememberConversation(hstring const& preview) {
    auto id=conversation_.Text(); if(!tektalk::LooksLikeUuid(id.c_str()))return;
    recent_.erase(std::remove_if(recent_.begin(),recent_.end(),[&](auto const& item){return item.id==id;}),recent_.end());
    recent_.insert(recent_.begin(),RecentChat{id,recipient_.Text(),preview}); ShowSidebar(activeTab_);
}
void MainWindow::SelectTab(uint32_t index) {
    activeTab_=index;
    for(uint32_t i=0;i<tabs_.size();++i) {
        auto selected=i==index;
        auto background=Brush(selected?L"TekPrimaryContainerBrush":L"TekSurfaceBrush");
        auto foreground=Brush(selected?L"TekOnPrimaryContainerBrush":L"TekMutedBrush");
        tabs_[i].Background(background); tabs_[i].Foreground(foreground);
        for(auto key:{L"ButtonBackgroundPointerOver",L"ButtonBackgroundPressed"})tabs_[i].Resources().Insert(box_value(key),background);
        for(auto key:{L"ButtonForegroundPointerOver",L"ButtonForegroundPressed"})tabs_[i].Resources().Insert(box_value(key),foreground);
    }
}
void MainWindow::ShowMessage() {
    SelectTab(0); ShowSidebar(0); content_.Children().Clear();
    if(messageContent_) { content_.Children().Append(messageContent_); return; }
    messageContent_=Grid();
    for(int i=0;i<5;++i) { RowDefinition row; row.Height(i==3?GridLengthHelper::FromValueAndType(1,GridUnitType::Star):GridLengthHelper::Auto()); messageContent_.RowDefinitions().Append(row); }
    auto heading=Text(Tr(L"Message"),28,true); heading.Margin({24,20,24,12}); messageContent_.Children().Append(heading);
    auto setup=StackPanel(); setup.Padding({24,0,24,8}); setup.Spacing(8); Grid::SetRow(setup,1);
    conversation_=TextBox(); Aid(conversation_,L"chat.conversation"); conversation_.Header(box_value(Tr(L"ConversationUuid")));
    recipient_=TextBox(); Aid(recipient_,L"chat.recipient"); recipient_.Header(box_value(Tr(L"RecipientUuid")));
    setup.Children().Append(conversation_); setup.Children().Append(recipient_);
    auto sync=Button(); Aid(sync,L"chat.sync"); sync.Content(box_value(Tr(L"SyncHistory"))); sync.Click([this](auto&&,auto&&){LoadMessages();}); setup.Children().Append(sync); messageContent_.Children().Append(setup);
    status_=Text(Tr(L"MessageStartHint"),12); status_.Foreground(Brush(L"TekMutedBrush")); status_.Margin({24,0,24,8}); Grid::SetRow(status_,2); messageContent_.Children().Append(status_);
    messages_=ListView(); messages_.Margin({16,0,16,0}); messages_.SelectionMode(ListViewSelectionMode::None); Aid(messages_,L"chat.messages");
    Style messageStyle{xaml_typename<ListViewItem>()};
    Setter stretch; stretch.Property(Control::HorizontalContentAlignmentProperty()); stretch.Value(box_value(HorizontalAlignment::Stretch)); messageStyle.Setters().Append(stretch);
    messages_.ItemContainerStyle(messageStyle);
    Grid::SetRow(messages_,3); messageContent_.Children().Append(messages_);
    auto composer=Grid(); composer.Padding({16,16,16,16}); composer.Background(Brush(L"TekSurfaceBrush"));
    composer.ColumnDefinitions().Append(ColumnDefinition()); ColumnDefinition sendColumn; sendColumn.Width(GridLengthHelper::Auto()); composer.ColumnDefinitions().Append(sendColumn);
    draft_=TextBox(); Aid(draft_,L"chat.draft"); draft_.PlaceholderText(Tr(L"ComposeMessage")); draft_.VerticalAlignment(VerticalAlignment::Center); composer.Children().Append(draft_);
    auto send=Button(); Aid(send,L"chat.send"); send.Content(box_value(Tr(L"Send"))); Primary(send); send.Margin({8,0,0,0}); Grid::SetColumn(send,1);
    send.Click([this](auto&&,auto&&){SendMessage();}); composer.Children().Append(send); Grid::SetRow(composer,4); messageContent_.Children().Append(composer);
    content_.Children().Append(messageContent_);
}
void MainWindow::AppendMessage(hstring const& body,bool mine) {
    auto bubble=Border(); bubble.Padding({12,12,12,12}); bubble.Margin({0,4,0,4}); bubble.CornerRadius({18,18,18,18});
    bubble.MaxWidth(520); bubble.HorizontalAlignment(mine?HorizontalAlignment::Right:HorizontalAlignment::Left);
    bubble.Background(Brush(mine?L"TekPrimaryBrush":L"TekSurfaceBrush"));
    auto label=Text(body); label.TextWrapping(TextWrapping::Wrap); label.Foreground(Brush(mine?L"TekOnPrimaryBrush":L"TekTextBrush")); bubble.Child(label);
    messages_.Items().Append(bubble); messages_.ScrollIntoView(bubble);
}
IAsyncAction MainWindow::LoadMessages() {
    if(!tokens_||!tektalk::LooksLikeUuid(conversation_.Text().c_str())){status_.Text(Tr(L"InvalidConversation"));co_return;}
    auto id=conversation_.Text();
    try {
        auto list=co_await api_.History(tokens_->access_token,id); if(conversation_.Text()!=id)co_return;
        messages_.Items().Clear(); hstring preview;
        for(auto const& item:list){auto m=tektalk::ApiClient::ParseMessage(item.GetObject());AppendMessage(m.body,m.sender_id==tokens_->user_id);preview=m.body;}
        status_.Text(Tr(L"SyncedPrefix")+to_hstring(list.Size())+Tr(L"MessagesSuffix")); RememberConversation(preview);
    }catch(hresult_error const& e){status_.Text(e.message());}
}
IAsyncAction MainWindow::SendMessage() {
    if(!tokens_||!tektalk::LooksLikeUuid(conversation_.Text().c_str())||!tektalk::LooksLikeUuid(recipient_.Text().c_str())||draft_.Text().empty()){status_.Text(Tr(L"InvalidMessage"));co_return;}
    auto id=conversation_.Text();
    try {
        auto m=tektalk::ApiClient::ParseMessage(co_await api_.Send(tokens_->access_token,id,recipient_.Text(),draft_.Text()));
        if(conversation_.Text()!=id)co_return;
        AppendMessage(m.body,true); draft_.Text(L""); status_.Text(Tr(L"SentSnowflake")+to_hstring(m.id)); RememberConversation(m.body);
    }catch(hresult_error const& e){status_.Text(e.message());}
}
void MainWindow::ShowAI(){SelectTab(1);ShowSidebar(1);content_.Children().Clear();auto panel=StackPanel();panel.Padding({42});panel.Spacing(12);panel.Children().Append(Text(L"TEK AI",32,true));panel.Children().Append(Text(Tr(L"AiDescription"),16));content_.Children().Append(panel);}
void MainWindow::ShowMe(){SelectTab(2);ShowSidebar(2);content_.Children().Clear();auto panel=StackPanel();panel.Padding({42});panel.Spacing(12);panel.Children().Append(Text(Tr(L"Me"),32,true));if(tokens_){panel.Children().Append(Text(Tr(L"User")+L": "+tokens_->user_id));panel.Children().Append(Text(Tr(L"Device")+L": "+tokens_->device_id));}panel.Children().Append(Text(core_.Available()?Tr(L"RustCoreConnectedLabel"):Tr(L"RustCoreFallbackLabel")));auto logout=Button();Aid(logout,L"auth.logout");logout.Content(box_value(Tr(L"SignOut")));logout.Click([this](auto&&,auto&&){tokens_.reset();challenge_=L"";register_mode_=false;ShowAuth();});panel.Children().Append(logout);content_.Children().Append(panel);}
}
