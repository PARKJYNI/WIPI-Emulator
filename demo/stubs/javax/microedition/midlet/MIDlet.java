// 컴파일용 스텁 — 런타임에는 에뮬레이터(wie_midp)가 실제 구현을 제공한다.
package javax.microedition.midlet;

public abstract class MIDlet {
    protected MIDlet() {}
    protected abstract void startApp();
    protected abstract void pauseApp();
    protected abstract void destroyApp(boolean unconditional);
    public final void notifyDestroyed() {}
}
