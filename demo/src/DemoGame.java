// WIPI 에뮬레이터 내장 데모 게임 "하트 캐치".
// 목적: 심사관·첫 사용자가 게임 파일 없이도 에뮬 기능(렌더링/키패드/게임 루프)을 확인.
// 좌우(◀▶ 또는 4/6)로 바구니를 움직여 떨어지는 하트를 받는다. MIT — 이 저장소 소유.

import javax.microedition.lcdui.Canvas;
import javax.microedition.lcdui.Display;
import javax.microedition.lcdui.Graphics;
import javax.microedition.midlet.MIDlet;

public class DemoGame extends MIDlet implements Runnable {
    private GameScreen screen;
    private boolean running;

    protected void startApp() {
        if (screen == null) {
            screen = new GameScreen();
            Display.getDisplay(this).setCurrent(screen);
            running = true;
            new Thread(this).start();
        }
    }

    protected void pauseApp() {}

    protected void destroyApp(boolean unconditional) {
        running = false;
    }

    public void run() {
        while (running) {
            screen.tick();
            screen.repaint();
            screen.serviceRepaints();
            try {
                Thread.sleep(50); // ~20fps
            } catch (InterruptedException e) {
                return;
            }
        }
    }

    static class GameScreen extends Canvas {
        private static final int W = 240;
        private static final int H = 320;

        // 자체 LCG — 고정핀 RustJava(62cf0c6a)에 Random.nextInt(int)가 없어서 (upstream엔 이후 추가됨)
        private long seed = System.currentTimeMillis();

        private int basketX = W / 2;
        private int dir = 0; // -1 좌, 1 우
        private int heartX = 60;
        private int heartY = 40;
        private int score = 0;
        private int missed = 0;
        private int frame = 0;

        protected void paint(Graphics g) {
            // 배경 (앱 아이콘과 같은 청록)
            g.setColor(0x128C7F);
            g.fillRect(0, 0, W, H);

            // 타이틀
            g.setColor(0xFFFFFF);
            g.drawString("WIPI EMULATOR DEMO", W / 2, 8, Graphics.TOP | Graphics.HCENTER);
            g.setColor(0xBFE8E2);
            g.drawString("방향키로 하트를 받아보세요", W / 2, 26, Graphics.TOP | Graphics.HCENTER);

            // 점수 — 문자열 연결(+)은 javac가 StringBuilder로 컴파일하는데
            // RustJava에는 StringBuffer만 있어 명시적으로 사용 (피처폰 시절 javac와 동일 산출)
            g.setColor(0xFFE3C2);
            g.drawString(text("SCORE ", score), 8, 48, Graphics.TOP | Graphics.LEFT);
            g.drawString(text("MISS ", missed), W - 70, 48, Graphics.TOP | Graphics.LEFT);

            // 하트 (픽셀풍: 원 2개 + 삼각형 대신 사각형 조합)
            g.setColor(0xFF4D6D);
            g.fillArc(heartX - 8, heartY - 6, 9, 9, 0, 360);
            g.fillArc(heartX, heartY - 6, 9, 9, 0, 360);
            g.fillRect(heartX - 7, heartY - 1, 15, 7);
            g.fillRect(heartX - 4, heartY + 6, 9, 4);
            g.fillRect(heartX - 1, heartY + 10, 3, 3);

            // 바구니 (주황)
            g.setColor(0xFF8A3D);
            g.fillRoundRect(basketX - 20, H - 40, 40, 14, 4, 4);
            g.setColor(0xD9661F);
            g.fillRect(basketX - 20, H - 40, 40, 3);

            // 바닥
            g.setColor(0x0E6E64);
            g.fillRect(0, H - 20, W, 20);
        }

        private int nextRandom(int bound) {
            seed = seed * 6364136223846793005L + 1442695040888963407L;
            int v = (int) (seed >>> 33) % bound;
            return v < 0 ? v + bound : v;
        }

        private static String text(String label, int n) {
            return new StringBuffer(label).append(n).toString();
        }

        void tick() {
            frame++;

            basketX += dir * 6;
            if (basketX < 20) basketX = 20;
            if (basketX > W - 20) basketX = W - 20;

            heartY += 4 + score / 5; // 점수 오르면 점점 빨라짐
            if (heartY > H - 44) {
                boolean caught = heartX > basketX - 26 && heartX < basketX + 26;
                if (caught) {
                    score++;
                } else {
                    missed++;
                }
                heartY = 40;
                heartX = 20 + nextRandom(W - 40);
            }
        }

        protected void keyPressed(int keyCode) {
            int action = getGameAction(keyCode);
            if (action == LEFT || keyCode == '4') dir = -1;
            else if (action == RIGHT || keyCode == '6') dir = 1;
        }

        protected void keyReleased(int keyCode) {
            int action = getGameAction(keyCode);
            if (action == LEFT || action == RIGHT || keyCode == '4' || keyCode == '6') dir = 0;
        }
    }
}
