package vn.tektalk
import org.bouncycastle.crypto.agreement.X25519Agreement
import org.bouncycastle.crypto.generators.X25519KeyPairGenerator
import org.bouncycastle.crypto.params.*
import org.bouncycastle.crypto.digests.SHA256Digest
import org.bouncycastle.crypto.macs.HMac
import org.bouncycastle.crypto.params.KeyParameter
import java.nio.ByteBuffer
import java.nio.ByteOrder
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec

data class SessionKeys(val publicKey:ByteArray,val key:ByteArray)
object TEKProtocol{
 fun derive(serverPublic:ByteArray):SessionKeys{val gen=X25519KeyPairGenerator();gen.init(X25519KeyGenerationParameters(java.security.SecureRandom()));val pair=gen.generateKeyPair();val priv=pair.private as X25519PrivateKeyParameters;val pub=pair.public as X25519PublicKeyParameters;val agree=X25519Agreement();agree.init(priv);val shared=ByteArray(32);agree.calculateAgreement(X25519PublicKeyParameters(serverPublic,0),shared,0);return SessionKeys(pub.encoded,hkdf(shared,"tektalk-v1".toByteArray(),"realtime-session".toByteArray()))}
 fun encrypt(key:ByteArray,kind:Int,session:Long,message:Long,sequence:Long,payload:ByteArray):ByteArray{val h=ByteBuffer.allocate(32).order(ByteOrder.BIG_ENDIAN).put(1).put(kind.toByte()).putShort(0).putLong(session).putLong(message).putLong(sequence).putInt(payload.size).array();val c=Cipher.getInstance("ChaCha20-Poly1305");c.init(Cipher.ENCRYPT_MODE,SecretKeySpec(key,"ChaCha20"),IvParameterSpec(nonce(sequence,message)));c.updateAAD(h);return h+c.doFinal(payload)}
 private fun nonce(seq:Long,msg:Long)=ByteBuffer.allocate(12).order(ByteOrder.BIG_ENDIAN).putLong(seq).putInt(msg.toInt()).array()
 private fun hkdf(ikm:ByteArray,salt:ByteArray,info:ByteArray):ByteArray{fun mac(k:ByteArray,d:ByteArray):ByteArray{val h=HMac(SHA256Digest());h.init(KeyParameter(k));h.update(d,0,d.size);return ByteArray(32).also{h.doFinal(it,0)}};val prk=mac(salt,ikm);return mac(prk,info+byteArrayOf(1))}
}
