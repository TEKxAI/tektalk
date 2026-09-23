package vn.tektalk
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest
import java.security.SecureRandom
import org.bouncycastle.crypto.engines.AESEngine
import org.bouncycastle.crypto.params.KeyParameter

data class MtMessage(val salt:Long,val sessionId:Long,val messageId:Long,val sequenceNo:Int,val body:ByteArray)
object MTProto2 {
 fun authKeyId(a:ByteArray)=MessageDigest.getInstance("SHA-1").digest(a).copyOfRange(12,20)
 fun encrypt(a:ByteArray,m:MtMessage,c2s:Boolean=true):ByteArray{require(a.size==256);val b=ByteBuffer.allocate(32+m.body.size+32).order(ByteOrder.LITTLE_ENDIAN).putLong(m.salt).putLong(m.sessionId).putLong(m.messageId).putInt(m.sequenceNo).putInt(m.body.size).put(m.body);var p=16-b.position()%16;if(p<12)p+=16;val plain=ByteArray(b.position()+p);b.flip();b.get(plain,0,b.remaining());val random=ByteArray(p);SecureRandom().nextBytes(random);random.copyInto(plain,plain.size-p);val x=if(c2s)0 else 8;val mk=sha(a.copyOfRange(88+x,120+x)+plain).copyOfRange(8,24);val(k,iv)=derive(a,mk,x);return authKeyId(a)+mk+ige(plain,k,iv,true)}
 private fun derive(a:ByteArray,m:ByteArray,x:Int):Pair<ByteArray,ByteArray>{val sa=sha(m+a.copyOfRange(x,x+36));val sb=sha(a.copyOfRange(40+x,76+x)+m);return Pair(sa.copyOfRange(0,8)+sb.copyOfRange(8,24)+sa.copyOfRange(24,32),sb.copyOfRange(0,8)+sa.copyOfRange(8,24)+sb.copyOfRange(24,32))}
 private fun ige(input:ByteArray,key:ByteArray,iv:ByteArray,enc:Boolean):ByteArray{val aes=AESEngine.newInstance();aes.init(enc,KeyParameter(key));var c=iv.copyOfRange(0,16);var p=iv.copyOfRange(16,32);val out=ByteArray(input.size);for(o in input.indices step 16){val block=input.copyOfRange(o,o+16);val mixed=ByteArray(16){i->block[i] xor if(enc)c[i] else p[i]};val crypt=ByteArray(16);aes.processBlock(mixed,0,crypt,0);val result=ByteArray(16){i->crypt[i] xor if(enc)p[i] else c[i]};result.copyInto(out,o);if(enc){c=result;p=block}else{p=result;c=block}};return out}
 private fun sha(v:ByteArray)=MessageDigest.getInstance("SHA-256").digest(v)
}
