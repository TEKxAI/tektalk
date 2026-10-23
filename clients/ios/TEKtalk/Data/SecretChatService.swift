import Foundation

@MainActor final class SecretChatService {
    private let api:APIClient
    private let tokens:Tokens
    private let core:SecretDevice
    private var policies:[UUID:SecretPolicy]=[:]
    private let encoder=JSONEncoder(),decoder=JSONDecoder()

    init?(tokens:Tokens,api:APIClient=APIClient()){
        guard let core=NativeCore.shared.secretDevice(userID:tokens.user_id,deviceID:tokens.device_id) else{return nil}
        self.tokens=tokens;self.api=api;self.core=core
    }
    private func register()async throws{
        guard let value=core.publicBundleJSON,let data=value.data(using:.utf8)else{throw URLError(.cannotDecodeContentData)}
        let bundle=try decoder.decode(CoreBundle.self,from:data)
        let _:ServerBundle=try await api.post("v1/secret/devices/register",body:RegisterSecretBundle(suite:bundle.suites[0],x25519_public:bundle.x25519_public,ml_kem_768_public:bundle.ml_kem_768_public),token:tokens.access_token)
    }
    func enable(conversation:UUID,kind:String,members:[UUID])async throws{
        try await register()
        let policy:SecretPolicy=try await api.post("v1/secret/conversations/enable",body:EnableSecretRequest(conversation_id:conversation,kind:kind,member_ids:members),token:tokens.access_token)
        let bundles:[ServerBundle]=try await api.post("v1/secret/conversations/bundles",body:SecretConversationRequest(conversation_id:conversation),token:tokens.access_token)
        guard bundles.contains(where:{$0.user_id != tokens.user_id})else{throw NSError(domain:"TEKtalk.SecretChat",code:1,userInfo:[NSLocalizedDescriptionKey:"Every peer must register a Secret Chat device before opt-in"])}
        let coreBundles=bundles.map{CoreBundle(user_id:$0.user_id.uuidString,device_id:$0.device_id.uuidString,x25519_public:$0.x25519_public,ml_kem_768_public:$0.ml_kem_768_public,suites:[$0.suite])}
        let bundleJSON=String(data:try encoder.encode(coreBundles),encoding:.utf8)!
        guard let wrappedJSON=core.begin(conversation:conversation.uuidString,kind:policy.kind,epoch:UInt64(policy.epoch),bundles:bundleJSON),let wrappedData=wrappedJSON.data(using:.utf8)else{throw URLError(.cannotDecodeContentData)}
        let wrapped=try decoder.decode([WrappedSenderKey].self,from:wrappedData)
        let request=PublishKeysRequest(conversation_id:conversation,epoch:policy.epoch,packets:wrapped.compactMap{guard let id=UUID(uuidString:$0.recipient_device_id)else{return nil};return PublishKeyPacket(recipient_device_id:id,packet:$0)})
        let _:StatusResponse=try await api.post("v1/secret/keys/publish",body:request,token:tokens.access_token);policies[conversation]=policy
    }
    private func prepare(_ conversation:UUID)async throws->SecretPolicy{
        try await register()
        let policy:SecretPolicy=try await api.post("v1/secret/conversations/policy",body:SecretConversationRequest(conversation_id:conversation),token:tokens.access_token)
        let packets:[KeyPacket]=try await api.post("v1/secret/keys/inbox",body:SecretConversationRequest(conversation_id:conversation),token:tokens.access_token)
        for packet in packets{let wrapped=String(data:try encoder.encode(packet.packet),encoding:.utf8)!;guard core.accept(conversation:conversation.uuidString,sender:packet.sender_device_id.uuidString,kind:policy.kind,epoch:UInt64(policy.epoch),wrapped:wrapped)else{throw URLError(.cannotDecodeContentData)}}
        policies[conversation]=policy;return policy
    }
    func history(conversation:UUID)async throws->[ChatMessage]{
        _=try await prepare(conversation)
        let rows:[SecretMessage]=try await api.post("v1/secret/messages/history",body:SecretConversationRequest(conversation_id:conversation),token:tokens.access_token)
        return try rows.map{row in let wire=CoreEncryptedMessage(conversation_id:row.conversation_id.uuidString,epoch:UInt64(row.epoch),sender_device_id:row.sender_device_id.uuidString,counter:UInt64(row.counter),nonce:row.nonce,ciphertext:row.ciphertext);let json=String(data:try encoder.encode(wire),encoding:.utf8)!;guard let body=core.decrypt(message:json)else{throw URLError(.cannotDecodeContentData)};return ChatMessage(id:row.id,conversation_id:row.conversation_id,sender_id:row.sender_id,recipient_id:row.sender_id,client_message_id:row.client_message_id,body:body,created_at:row.created_at)}
    }
    func send(conversation:UUID,text:String)async throws->ChatMessage{
        let policy:SecretPolicy
        if let cached=policies[conversation]{policy=cached}else{policy=try await prepare(conversation)}
        guard let json=core.encrypt(conversation:conversation.uuidString,epoch:UInt64(policy.epoch),plaintext:text),let data=json.data(using:.utf8)else{throw URLError(.cannotDecodeContentData)}
        let wire=try decoder.decode(CoreEncryptedMessage.self,from:data)
        let response:SendSecretResponse=try await api.post("v1/secret/messages/send",body:SendSecretRequest(conversation_id:conversation,client_message_id:UUID(),epoch:Int64(wire.epoch),counter:Int64(wire.counter),nonce:wire.nonce,ciphertext:wire.ciphertext),token:tokens.access_token)
        let row=response.message;return ChatMessage(id:row.id,conversation_id:row.conversation_id,sender_id:row.sender_id,recipient_id:row.sender_id,client_message_id:row.client_message_id,body:text,created_at:row.created_at)
    }
}
