#import <Cocoa/Cocoa.h>

int main() {
    NSImageView *v = [[NSImageView alloc] init];
    [v setTag:101];
    NSLog(@"Tag: %ld", (long)[v tag]);
    return 0;
}
