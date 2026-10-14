const encoder=new TextEncoder();
const decoder=new TextDecoder();
const requestResult=request=>new Promise((resolve,reject)=>{request.onsuccess=()=>resolve(request.result);request.onerror=()=>reject(request.error)});
const transactionDone=transaction=>new Promise((resolve,reject)=>{transaction.oncomplete=resolve;transaction.onerror=()=>reject(transaction.error);transaction.onabort=()=>reject(transaction.error)});

/** Web-specific encrypted IndexedDB adapter. Native clients use Rust/SQLCipher. */
class EncryptedWebStore{
  constructor(accountId,db,key){this.accountId=accountId;this.db=db;this.key=key}
  static async open(accountId){
    if(!/^[1-9][0-9]*$/.test(String(accountId)))throw new Error('Invalid account ID');
    const request=indexedDB.open(`tektalk.account.${accountId}`,1);
    request.onupgradeneeded=()=>{
      const db=request.result;
      if(!db.objectStoreNames.contains('system'))db.createObjectStore('system');
      if(!db.objectStoreNames.contains('records'))db.createObjectStore('records');
    };
    const db=await requestResult(request);
    const readTransaction=db.transaction('system','readonly');
    let key=await requestResult(readTransaction.objectStore('system').get('database-key'));
    await transactionDone(readTransaction);
    if(!key){
      key=await crypto.subtle.generateKey({name:'AES-GCM',length:256},false,['encrypt','decrypt']);
      const writeTransaction=db.transaction('system','readwrite');
      writeTransaction.objectStore('system').add(key,'database-key');
      await transactionDone(writeTransaction);
    }
    return new EncryptedWebStore(String(accountId),db,key);
  }
  async put(namespace,id,value){
    const recordKey=`${namespace}:${id}`;
    const iv=crypto.getRandomValues(new Uint8Array(12));
    const additionalData=encoder.encode(`${this.accountId}:${recordKey}`);
    const plaintext=encoder.encode(JSON.stringify(value));
    const ciphertext=await crypto.subtle.encrypt({name:'AES-GCM',iv,additionalData},this.key,plaintext);
    const transaction=this.db.transaction('records','readwrite');
    transaction.objectStore('records').put({iv:[...iv],ciphertext:[...new Uint8Array(ciphertext)]},recordKey);
    await transactionDone(transaction);
  }
  async get(namespace,id){
    const recordKey=`${namespace}:${id}`;
    const transaction=this.db.transaction('records','readonly');
    const encrypted=await requestResult(transaction.objectStore('records').get(recordKey));
    await transactionDone(transaction);
    if(!encrypted)return null;
    const plaintext=await crypto.subtle.decrypt({name:'AES-GCM',iv:new Uint8Array(encrypted.iv),additionalData:encoder.encode(`${this.accountId}:${recordKey}`)},this.key,new Uint8Array(encrypted.ciphertext));
    return JSON.parse(decoder.decode(plaintext));
  }
  close(){this.db.close()}
}

window.EncryptedWebStore=EncryptedWebStore;
