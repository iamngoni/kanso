// Kanso spike host: the UIKit entry point for gpui-pre-mobile.
//
// Adapted from gpui-mobile's example/ios/main.m. Two changes: iOS 27's SDK
// refuses to launch an app without the UIScene lifecycle, so GPUI is started
// from a scene delegate; and gpui-mobile makes its UIWindow with
// -initWithFrame:, so that call is redirected to -initWithWindowScene: for the
// scene that connected.

#import <UIKit/UIKit.h>
#import <objc/runtime.h>
#import "gpui_ios.h"

void gpui_ios_register_app(void);

static UIWindowScene *KansoScene;

@interface UIWindow (KansoScene)
@end

@implementation UIWindow (KansoScene)
- (instancetype)kanso_initWithFrame:(CGRect)frame {
    if (KansoScene) {
        UIWindow *window = [self initWithWindowScene:KansoScene];
        window.frame = frame;
        return window;
    }
    return [self kanso_initWithFrame:frame];  // swizzled: the original
}
@end

@interface KansoAppDelegate : UIResponder <UIApplicationDelegate>
@property (nonatomic, strong) CADisplayLink *displayLink;
@property (nonatomic, assign) void *gpuiWindow;
@end

static KansoAppDelegate *Delegate;
static void KansoResumeFrames(void *context);

@implementation KansoAppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    Delegate = self;
    Method a = class_getInstanceMethod([UIWindow class], @selector(initWithFrame:));
    Method b = class_getInstanceMethod([UIWindow class], @selector(kanso_initWithFrame:));
    method_exchangeImplementations(a, b);
    return YES;
}

- (UISceneConfiguration *)application:(UIApplication *)application
    configurationForConnectingSceneSession:(UISceneSession *)session
                                   options:(UISceneConnectionOptions *)options {
    UISceneConfiguration *config = [[UISceneConfiguration alloc] initWithName:@"Default" sessionRole:session.role];
    config.delegateClass = NSClassFromString(@"KansoSceneDelegate");
    return config;
}

- (void)startGpuiInScene:(UIWindowScene *)scene {
    KansoScene = scene;
    gpui_ios_register_app();
    gpui_ios_run_demo();
    self.gpuiWindow = gpui_ios_get_window();
    if (!self.gpuiWindow) {
        NSLog(@"kanso: no GPUI window was created");
        return;
    }
    // GPUI says after each frame whether it wants another; the link pauses in
    // between and the waker resumes it, so an idle screen costs no CPU.
    self.displayLink = [CADisplayLink displayLinkWithTarget:self selector:@selector(renderFrame)];
    [self.displayLink addToRunLoop:[NSRunLoop mainRunLoop] forMode:NSRunLoopCommonModes];
    gpui_ios_set_frame_waker(self.gpuiWindow, KansoResumeFrames, (__bridge void *)self);
}

- (void)renderFrame {
    if (self.gpuiWindow && !gpui_ios_request_frame(self.gpuiWindow)) {
        self.displayLink.paused = YES;
    }
}

static void KansoResumeFrames(void *context) {
    ((__bridge KansoAppDelegate *)context).displayLink.paused = NO;
}

@end

@interface KansoSceneDelegate : UIResponder <UIWindowSceneDelegate>
@property (nonatomic, strong) UIWindow *window;
@end

@implementation KansoSceneDelegate
- (void)scene:(UIScene *)scene willConnectToSession:(UISceneSession *)session options:(UISceneConnectionOptions *)options {
    if (![scene isKindOfClass:[UIWindowScene class]]) return;
    [Delegate startGpuiInScene:(UIWindowScene *)scene];
}
- (void)sceneDidBecomeActive:(UIScene *)scene { gpui_ios_did_become_active(NULL); }
- (void)sceneWillResignActive:(UIScene *)scene { gpui_ios_will_resign_active(NULL); }
@end

int main(int argc, char *argv[]) {
    @autoreleasepool {
        return UIApplicationMain(argc, argv, nil, NSStringFromClass([KansoAppDelegate class]));
    }
}
