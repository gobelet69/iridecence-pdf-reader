#import <Cocoa/Cocoa.h>

int main() {
    NSProgressIndicator *v = [[NSProgressIndicator alloc] init];
    [v setTag:102];
    NSLog(@"Tag: %ld", (long)[v tag]);
    return 0;
}
