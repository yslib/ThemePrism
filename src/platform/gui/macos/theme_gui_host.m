#import <Cocoa/Cocoa.h>

typedef char *(*ThemeCopySnapshotFn)(void *);
typedef void (*ThemeDispatchCommandFn)(void *, const char *);
typedef void (*ThemeFreeStringFn)(char *);

@protocol ThemeSliderTracking <NSObject>

- (void)sliderInteractionDidEnd:(NSSlider *)sender;

@end

@interface ThemeTrackingSlider : NSSlider

@property (nonatomic, weak) id<ThemeSliderTracking> trackingDelegate;

@end

static NSColor *ThemeColorFromHex(NSString *hex) {
    NSString *value = [[hex stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]]
        stringByReplacingOccurrencesOfString:@"#" withString:@""];
    if (value.length != 6) {
        return [NSColor clearColor];
    }

    unsigned int rgb = 0;
    if (![[NSScanner scannerWithString:value] scanHexInt:&rgb]) {
        return [NSColor clearColor];
    }

    CGFloat red = ((rgb >> 16) & 0xFF) / 255.0;
    CGFloat green = ((rgb >> 8) & 0xFF) / 255.0;
    CGFloat blue = (rgb & 0xFF) / 255.0;
    return [NSColor colorWithSRGBRed:red green:green blue:blue alpha:1.0];
}

@interface ThemeGuiController : NSObject <NSApplicationDelegate, NSTableViewDataSource, NSTableViewDelegate, NSTextFieldDelegate, ThemeSliderTracking>

@property (nonatomic, assign) void *bridgeContext;
@property (nonatomic, assign) ThemeCopySnapshotFn copySnapshot;
@property (nonatomic, assign) ThemeDispatchCommandFn dispatchCommand;
@property (nonatomic, assign) ThemeFreeStringFn freeString;

@property (nonatomic, strong) NSWindow *window;
@property (nonatomic, strong) NSTableView *tokenTable;
@property (nonatomic, strong) NSStackView *paramsStack;
@property (nonatomic, strong) NSStackView *paletteStack;
@property (nonatomic, strong) NSTextView *previewTextView;
@property (nonatomic, strong) NSTextField *statusLabel;
@property (nonatomic, strong) NSTextField *inspectorTitleLabel;
@property (nonatomic, strong) NSTextField *inspectorSummaryLabel;
@property (nonatomic, strong) NSView *inspectorSwatch;
@property (nonatomic, strong) NSStackView *inspectorFieldsStack;
@property (nonatomic, strong) NSArray<NSDictionary *> *tokens;
@property (nonatomic, strong) NSDictionary *snapshot;
@property (nonatomic, assign) BOOL suppressTokenSelection;

- (instancetype)initWithContext:(void *)context
                   copySnapshot:(ThemeCopySnapshotFn)copySnapshot
                dispatchCommand:(ThemeDispatchCommandFn)dispatchCommand
                     freeString:(ThemeFreeStringFn)freeString;

@end

@implementation ThemeTrackingSlider

- (void)mouseDown:(NSEvent *)event {
    [super mouseDown:event];
    [self.trackingDelegate sliderInteractionDidEnd:self];
}

@end

@implementation ThemeGuiController

- (instancetype)initWithContext:(void *)context
                   copySnapshot:(ThemeCopySnapshotFn)copySnapshot
                dispatchCommand:(ThemeDispatchCommandFn)dispatchCommand
                     freeString:(ThemeFreeStringFn)freeString {
    self = [super init];
    if (self) {
        _bridgeContext = context;
        _copySnapshot = copySnapshot;
        _dispatchCommand = dispatchCommand;
        _freeString = freeString;
        _tokens = @[];
        _suppressTokenSelection = NO;
    }
    return self;
}

- (void)applicationDidFinishLaunching:(NSNotification *)notification {
    (void)notification;
    [self installMainMenu];
    [self buildWindow];
    [self refreshUI];
    [self.window makeKeyAndOrderFront:nil];
    [NSApp activateIgnoringOtherApps:YES];
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication *)sender {
    (void)sender;
    return YES;
}

- (void)installMainMenu {
    NSMenu *menuBar = [[NSMenu alloc] init];
    NSMenuItem *appMenuItem = [[NSMenuItem alloc] init];
    [menuBar addItem:appMenuItem];
    [NSApp setMainMenu:menuBar];

    NSMenu *appMenu = [[NSMenu alloc] init];
    NSString *processName = NSProcessInfo.processInfo.processName;
    NSString *quitTitle = [NSString stringWithFormat:@"Quit %@", processName];
    NSMenuItem *quitItem = [[NSMenuItem alloc] initWithTitle:quitTitle
                                                      action:@selector(terminate:)
                                               keyEquivalent:@"q"];
    [appMenu addItem:quitItem];
    [appMenuItem setSubmenu:appMenu];
}

- (void)buildWindow {
    NSRect frame = NSMakeRect(0, 0, 1380, 860);
    NSWindowStyleMask style = NSWindowStyleMaskTitled | NSWindowStyleMaskClosable |
                              NSWindowStyleMaskResizable | NSWindowStyleMaskMiniaturizable;
    self.window = [[NSWindow alloc] initWithContentRect:frame
                                              styleMask:style
                                                backing:NSBackingStoreBuffered
                                                  defer:NO];
    self.window.title = @"Theme Generator";
    self.window.titlebarAppearsTransparent = YES;
    self.window.toolbarStyle = NSWindowToolbarStyleUnified;
    self.window.minSize = NSMakeSize(1180, 760);

    NSView *contentView = self.window.contentView;

    NSSplitView *splitView = [[NSSplitView alloc] init];
    splitView.translatesAutoresizingMaskIntoConstraints = NO;
    splitView.vertical = YES;
    splitView.dividerStyle = NSSplitViewDividerStyleThin;

    NSScrollView *leftPane = [self buildSidebar];
    NSScrollView *centerPane = [self buildCenterPane];
    NSScrollView *rightPane = [self buildInspectorPane];

    [splitView addArrangedSubview:leftPane];
    [splitView addArrangedSubview:centerPane];
    [splitView addArrangedSubview:rightPane];

    [leftPane.widthAnchor constraintEqualToConstant:230.0].active = YES;
    [rightPane.widthAnchor constraintEqualToConstant:360.0].active = YES;

    NSVisualEffectView *statusBar = [[NSVisualEffectView alloc] init];
    statusBar.translatesAutoresizingMaskIntoConstraints = NO;
    statusBar.material = NSVisualEffectMaterialSidebar;
    statusBar.blendingMode = NSVisualEffectBlendingModeBehindWindow;

    self.statusLabel = [NSTextField labelWithString:@"Ready"];
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.statusLabel.font = [NSFont systemFontOfSize:12.0 weight:NSFontWeightRegular];
    self.statusLabel.textColor = NSColor.secondaryLabelColor;
    [statusBar addSubview:self.statusLabel];

    [contentView addSubview:splitView];
    [contentView addSubview:statusBar];

    [NSLayoutConstraint activateConstraints:@[
        [splitView.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [splitView.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [splitView.topAnchor constraintEqualToAnchor:contentView.topAnchor],
        [splitView.bottomAnchor constraintEqualToAnchor:statusBar.topAnchor],

        [statusBar.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [statusBar.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [statusBar.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],
        [statusBar.heightAnchor constraintEqualToConstant:34.0],

        [self.statusLabel.leadingAnchor constraintEqualToAnchor:statusBar.leadingAnchor constant:16.0],
        [self.statusLabel.trailingAnchor constraintEqualToAnchor:statusBar.trailingAnchor constant:-16.0],
        [self.statusLabel.centerYAnchor constraintEqualToAnchor:statusBar.centerYAnchor],
    ]];
}

- (NSScrollView *)buildSidebar {
    self.tokenTable = [[NSTableView alloc] init];
    self.tokenTable.headerView = nil;
    self.tokenTable.delegate = self;
    self.tokenTable.dataSource = self;
    self.tokenTable.rowHeight = 30.0;
    self.tokenTable.style = NSTableViewStyleSourceList;
    self.tokenTable.translatesAutoresizingMaskIntoConstraints = NO;

    NSTableColumn *column = [[NSTableColumn alloc] initWithIdentifier:@"tokens"];
    column.resizingMask = NSTableColumnAutoresizingMask;
    [self.tokenTable addTableColumn:column];

    NSScrollView *scrollView = [[NSScrollView alloc] init];
    scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    scrollView.documentView = self.tokenTable;
    scrollView.hasVerticalScroller = YES;
    scrollView.drawsBackground = NO;
    scrollView.automaticallyAdjustsContentInsets = YES;
    return scrollView;
}

- (NSScrollView *)buildCenterPane {
    NSStackView *stack = [[NSStackView alloc] init];
    stack.orientation = NSUserInterfaceLayoutOrientationVertical;
    stack.alignment = NSLayoutAttributeLeading;
    stack.spacing = 18.0;
    stack.edgeInsets = NSEdgeInsetsMake(20.0, 20.0, 20.0, 20.0);
    stack.translatesAutoresizingMaskIntoConstraints = NO;

    [stack addArrangedSubview:[self sectionLabel:@"Theme Parameters"]];
    self.paramsStack = [self verticalSectionStack];
    [stack addArrangedSubview:self.paramsStack];

    [stack addArrangedSubview:[self sectionLabel:@"Palette"]];
    self.paletteStack = [self verticalSectionStack];
    [stack addArrangedSubview:self.paletteStack];

    [stack addArrangedSubview:[self sectionLabel:@"Preview"]];
    NSScrollView *previewScroll = [[NSScrollView alloc] init];
    previewScroll.translatesAutoresizingMaskIntoConstraints = NO;
    previewScroll.hasVerticalScroller = YES;
    previewScroll.borderType = NSBezelBorder;
    previewScroll.drawsBackground = NO;

    self.previewTextView = [[NSTextView alloc] init];
    self.previewTextView.editable = NO;
    self.previewTextView.selectable = YES;
    self.previewTextView.richText = YES;
    self.previewTextView.drawsBackground = YES;
    self.previewTextView.minSize = NSMakeSize(100.0, 220.0);
    self.previewTextView.maxSize = NSMakeSize(CGFLOAT_MAX, CGFLOAT_MAX);
    self.previewTextView.verticallyResizable = YES;
    self.previewTextView.horizontallyResizable = NO;
    self.previewTextView.textContainerInset = NSMakeSize(12.0, 14.0);
    self.previewTextView.textContainer.widthTracksTextView = YES;
    previewScroll.documentView = self.previewTextView;
    [previewScroll.heightAnchor constraintEqualToConstant:250.0].active = YES;
    [stack addArrangedSubview:previewScroll];

    NSScrollView *scrollView = [[NSScrollView alloc] init];
    scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    scrollView.documentView = stack;
    scrollView.hasVerticalScroller = YES;
    scrollView.drawsBackground = NO;
    scrollView.automaticallyAdjustsContentInsets = YES;
    return scrollView;
}

- (NSScrollView *)buildInspectorPane {
    NSStackView *stack = [[NSStackView alloc] init];
    stack.orientation = NSUserInterfaceLayoutOrientationVertical;
    stack.alignment = NSLayoutAttributeLeading;
    stack.spacing = 18.0;
    stack.edgeInsets = NSEdgeInsetsMake(20.0, 20.0, 20.0, 20.0);
    stack.translatesAutoresizingMaskIntoConstraints = NO;

    [stack addArrangedSubview:[self sectionLabel:@"Inspector"]];
    self.inspectorTitleLabel = [NSTextField labelWithString:@"Token"];
    self.inspectorTitleLabel.font = [NSFont systemFontOfSize:20.0 weight:NSFontWeightSemibold];
    [stack addArrangedSubview:self.inspectorTitleLabel];

    NSStackView *colorRow = [[NSStackView alloc] init];
    colorRow.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    colorRow.alignment = NSLayoutAttributeCenterY;
    colorRow.spacing = 10.0;
    colorRow.translatesAutoresizingMaskIntoConstraints = NO;
    self.inspectorSwatch = [self swatchViewWithHex:@"#000000" size:NSMakeSize(28.0, 18.0) radius:6.0];
    [colorRow addArrangedSubview:self.inspectorSwatch];
    self.inspectorSummaryLabel = [NSTextField labelWithString:@"Rule summary"];
    self.inspectorSummaryLabel.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightRegular];
    self.inspectorSummaryLabel.textColor = NSColor.secondaryLabelColor;
    self.inspectorSummaryLabel.lineBreakMode = NSLineBreakByWordWrapping;
    self.inspectorSummaryLabel.maximumNumberOfLines = 2;
    [colorRow addArrangedSubview:self.inspectorSummaryLabel];
    [stack addArrangedSubview:colorRow];

    self.inspectorFieldsStack = [self verticalSectionStack];
    [stack addArrangedSubview:self.inspectorFieldsStack];

    [stack addArrangedSubview:[self sectionLabel:@"Actions"]];
    NSStackView *actions = [[NSStackView alloc] init];
    actions.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    actions.spacing = 10.0;
    actions.distribution = NSStackViewDistributionFillEqually;
    actions.translatesAutoresizingMaskIntoConstraints = NO;
    [actions addArrangedSubview:[self actionButtonWithTitle:@"Save" command:@"save"]];
    [actions addArrangedSubview:[self actionButtonWithTitle:@"Load" command:@"load"]];
    [actions addArrangedSubview:[self actionButtonWithTitle:@"Export" command:@"export"]];
    [actions addArrangedSubview:[self actionButtonWithTitle:@"Reset" command:@"reset"]];
    [stack addArrangedSubview:actions];

    NSScrollView *scrollView = [[NSScrollView alloc] init];
    scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    scrollView.documentView = stack;
    scrollView.hasVerticalScroller = YES;
    scrollView.drawsBackground = NO;
    scrollView.automaticallyAdjustsContentInsets = YES;
    return scrollView;
}

- (void)refreshUI {
    NSDictionary *snapshot = [self fetchSnapshot];
    if (snapshot == nil) {
        return;
    }
    [self applySnapshot:snapshot rebuildControls:YES preservingSliderId:nil];
}

- (NSDictionary *)fetchSnapshot {
    char *raw = self.copySnapshot(self.bridgeContext);
    if (raw == NULL) {
        return nil;
    }

    NSData *data = [NSData dataWithBytes:raw length:strlen(raw)];
    self.freeString(raw);

    NSError *error = nil;
    id object = [NSJSONSerialization JSONObjectWithData:data options:0 error:&error];
    if (error != nil || ![object isKindOfClass:[NSDictionary class]]) {
        return nil;
    }

    return (NSDictionary *)object;
}

- (NSInteger)selectedTokenIndex {
    for (NSUInteger index = 0; index < self.tokens.count; index += 1) {
        NSDictionary *token = self.tokens[index];
        if ([token[@"selected"] boolValue]) {
            return (NSInteger)index;
        }
    }
    return 0;
}

- (void)rebuildParams:(NSArray<NSDictionary *> *)params {
    [self clearStack:self.paramsStack];
    for (NSDictionary *field in params) {
        [self.paramsStack addArrangedSubview:[self scalarControlRowForField:field]];
    }
}

- (void)rebuildPalette:(NSArray<NSDictionary *> *)palette {
    [self clearStack:self.paletteStack];
    for (NSDictionary *item in palette) {
        NSString *label = item[@"label"] ?: @"";
        NSString *hex = item[@"color_hex"] ?: @"#000000";
        NSStackView *row = [[NSStackView alloc] init];
        row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
        row.alignment = NSLayoutAttributeCenterY;
        row.spacing = 10.0;

        [row addArrangedSubview:[self swatchViewWithHex:hex size:NSMakeSize(26.0, 14.0) radius:4.0]];

        NSTextField *textLabel = [NSTextField labelWithString:label];
        textLabel.font = [NSFont systemFontOfSize:12.0 weight:NSFontWeightRegular];
        [row addArrangedSubview:textLabel];

        NSTextField *valueLabel = [NSTextField labelWithString:hex];
        valueLabel.font = [NSFont monospacedSystemFontOfSize:11.5 weight:NSFontWeightRegular];
        valueLabel.textColor = NSColor.secondaryLabelColor;
        [row addArrangedSubview:valueLabel];

        [self.paletteStack addArrangedSubview:row];
    }
}

- (void)rebuildPreview:(NSDictionary *)snapshot {
    NSDictionary *theme = snapshot[@"theme"] ?: @{};
    NSString *backgroundHex = theme[@"background_hex"] ?: @"#FFFFFF";
    self.previewTextView.backgroundColor = ThemeColorFromHex(backgroundHex);

    NSMutableAttributedString *content = [[NSMutableAttributedString alloc] init];
    NSArray<NSDictionary *> *lines = snapshot[@"preview"] ?: @[];
    for (NSDictionary *line in lines) {
        NSArray<NSDictionary *> *segments = line[@"segments"] ?: @[];
        for (NSDictionary *segment in segments) {
            NSString *text = segment[@"text"] ?: @"";
            NSString *foregroundHex = segment[@"foreground_hex"] ?: @"#000000";
            NSDictionary *attributes = @{
                NSForegroundColorAttributeName: ThemeColorFromHex(foregroundHex),
                NSFontAttributeName: [NSFont monospacedSystemFontOfSize:13.0 weight:NSFontWeightRegular],
            };
            [content appendAttributedString:[[NSAttributedString alloc] initWithString:text attributes:attributes]];
        }
        [content appendAttributedString:[[NSAttributedString alloc] initWithString:@"\n"]];
    }

    [[self.previewTextView textStorage] setAttributedString:content];
}

- (void)rebuildInspector:(NSDictionary *)inspector {
    NSString *tokenLabel = inspector[@"token_label"] ?: @"Token";
    NSString *tokenHex = inspector[@"token_color_hex"] ?: @"#000000";
    NSString *summary = inspector[@"rule_summary"] ?: @"";

    self.inspectorTitleLabel.stringValue = tokenLabel;
    [self configureSwatchView:self.inspectorSwatch withHex:tokenHex];
    self.inspectorSummaryLabel.stringValue = summary;

    [self clearStack:self.inspectorFieldsStack];
    NSArray<NSDictionary *> *fields = inspector[@"fields"] ?: @[];
    for (NSDictionary *field in fields) {
        NSString *kind = field[@"kind"] ?: @"";
        if ([kind isEqualToString:@"choice"]) {
            [self.inspectorFieldsStack addArrangedSubview:[self choiceControlRowForField:field]];
        } else if ([kind isEqualToString:@"scalar"]) {
            [self.inspectorFieldsStack addArrangedSubview:[self scalarControlRowForField:field]];
        } else if ([kind isEqualToString:@"color"]) {
            [self.inspectorFieldsStack addArrangedSubview:[self colorControlRowForField:field]];
        }
    }
}

- (NSView *)scalarControlRowForField:(NSDictionary *)field {
    NSString *label = field[@"label"] ?: @"";
    NSString *fieldId = field[@"id"] ?: @"";
    NSString *valueText = field[@"value_text"] ?: @"";

    NSStackView *row = [[NSStackView alloc] init];
    row.identifier = fieldId;
    row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    row.alignment = NSLayoutAttributeCenterY;
    row.spacing = 12.0;
    row.translatesAutoresizingMaskIntoConstraints = NO;

    NSTextField *labelView = [NSTextField labelWithString:label];
    labelView.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
    [labelView.widthAnchor constraintEqualToConstant:130.0].active = YES;
    [row addArrangedSubview:labelView];

    ThemeTrackingSlider *slider = [[ThemeTrackingSlider alloc] init];
    slider.identifier = fieldId;
    slider.minValue = [field[@"min"] doubleValue];
    slider.maxValue = [field[@"max"] doubleValue];
    slider.doubleValue = [field[@"current"] doubleValue];
    slider.target = self;
    slider.action = @selector(sliderChanged:);
    slider.continuous = YES;
    slider.trackingDelegate = self;
    [slider.widthAnchor constraintGreaterThanOrEqualToConstant:180.0].active = YES;
    [row addArrangedSubview:slider];

    NSTextField *valueView = [NSTextField labelWithString:valueText];
    valueView.identifier = @"ScalarValue";
    valueView.font = [NSFont monospacedSystemFontOfSize:11.5 weight:NSFontWeightRegular];
    valueView.textColor = NSColor.secondaryLabelColor;
    [valueView.widthAnchor constraintEqualToConstant:72.0].active = YES;
    [row addArrangedSubview:valueView];

    return row;
}

- (NSView *)choiceControlRowForField:(NSDictionary *)field {
    NSString *label = field[@"label"] ?: @"";
    NSString *fieldId = field[@"id"] ?: @"";
    NSString *selectedKey = field[@"selected_key"] ?: @"";

    NSStackView *row = [[NSStackView alloc] init];
    row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    row.alignment = NSLayoutAttributeCenterY;
    row.spacing = 12.0;

    NSTextField *labelView = [NSTextField labelWithString:label];
    labelView.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
    [labelView.widthAnchor constraintEqualToConstant:130.0].active = YES;
    [row addArrangedSubview:labelView];

    NSPopUpButton *popup = [[NSPopUpButton alloc] init];
    popup.identifier = fieldId;
    popup.target = self;
    popup.action = @selector(choiceChanged:);
    [popup.widthAnchor constraintGreaterThanOrEqualToConstant:200.0].active = YES;

    NSArray<NSDictionary *> *options = field[@"options"] ?: @[];
    NSInteger selectedIndex = 0;
    for (NSUInteger index = 0; index < options.count; index += 1) {
        NSDictionary *option = options[index];
        NSString *title = option[@"label"] ?: @"";
        NSString *key = option[@"key"] ?: @"";
        [popup addItemWithTitle:title];
        popup.lastItem.representedObject = key;
        if ([key isEqualToString:selectedKey]) {
            selectedIndex = (NSInteger)index;
        }
    }
    [popup selectItemAtIndex:selectedIndex];
    [row addArrangedSubview:popup];

    return row;
}

- (NSView *)colorControlRowForField:(NSDictionary *)field {
    NSString *label = field[@"label"] ?: @"";
    NSString *fieldId = field[@"id"] ?: @"";
    NSString *value = field[@"value_text"] ?: @"";
    NSString *hex = field[@"color_hex"] ?: @"#000000";

    NSStackView *row = [[NSStackView alloc] init];
    row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    row.alignment = NSLayoutAttributeCenterY;
    row.spacing = 12.0;

    NSTextField *labelView = [NSTextField labelWithString:label];
    labelView.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
    [labelView.widthAnchor constraintEqualToConstant:130.0].active = YES;
    [row addArrangedSubview:labelView];

    [row addArrangedSubview:[self swatchViewWithHex:hex size:NSMakeSize(26.0, 16.0) radius:5.0]];

    NSTextField *textField = [[NSTextField alloc] init];
    textField.identifier = fieldId;
    textField.stringValue = value;
    textField.placeholderString = @"#224466";
    textField.font = [NSFont monospacedSystemFontOfSize:12.0 weight:NSFontWeightRegular];
    textField.target = self;
    textField.action = @selector(textCommitted:);
    textField.delegate = self;
    [textField.widthAnchor constraintGreaterThanOrEqualToConstant:160.0].active = YES;
    [row addArrangedSubview:textField];

    return row;
}

- (NSButton *)actionButtonWithTitle:(NSString *)title command:(NSString *)command {
    NSButton *button = [NSButton buttonWithTitle:title target:self action:@selector(commandButtonPressed:)];
    button.identifier = command;
    button.bezelStyle = NSBezelStyleRounded;
    return button;
}

- (NSTextField *)sectionLabel:(NSString *)title {
    NSTextField *label = [NSTextField labelWithString:title];
    label.font = [NSFont systemFontOfSize:13.0 weight:NSFontWeightSemibold];
    label.textColor = NSColor.secondaryLabelColor;
    return label;
}

- (NSStackView *)verticalSectionStack {
    NSStackView *stack = [[NSStackView alloc] init];
    stack.orientation = NSUserInterfaceLayoutOrientationVertical;
    stack.alignment = NSLayoutAttributeLeading;
    stack.spacing = 10.0;
    stack.translatesAutoresizingMaskIntoConstraints = NO;
    return stack;
}

- (NSView *)swatchViewWithHex:(NSString *)hex size:(NSSize)size radius:(CGFloat)radius {
    NSView *view = [[NSView alloc] initWithFrame:NSMakeRect(0, 0, size.width, size.height)];
    [view.widthAnchor constraintEqualToConstant:size.width].active = YES;
    [view.heightAnchor constraintEqualToConstant:size.height].active = YES;
    view.wantsLayer = YES;
    view.layer.cornerRadius = radius;
    view.layer.borderWidth = 0.5;
    view.layer.borderColor = NSColor.separatorColor.CGColor;
    view.layer.backgroundColor = ThemeColorFromHex(hex).CGColor;
    return view;
}

- (void)configureSwatchView:(NSView *)view withHex:(NSString *)hex {
    view.wantsLayer = YES;
    view.layer.backgroundColor = ThemeColorFromHex(hex).CGColor;
}

- (void)clearStack:(NSStackView *)stack {
    NSArray<NSView *> *views = stack.arrangedSubviews.copy;
    for (NSView *view in views) {
        [stack removeArrangedSubview:view];
        [view removeFromSuperview];
    }
}

- (void)sliderChanged:(NSSlider *)sender {
    NSString *command = [NSString stringWithFormat:@"set-scalar|%@|%.6f", sender.identifier, sender.doubleValue];
    self.dispatchCommand(self.bridgeContext, command.UTF8String);
    [self updateScalarValueLabelForSlider:sender];
}

- (void)choiceChanged:(NSPopUpButton *)sender {
    NSString *value = sender.selectedItem.representedObject ?: @"";
    NSString *command = [NSString stringWithFormat:@"set-choice|%@|%@", sender.identifier, value];
    [self dispatchAndRefresh:command];
}

- (void)textCommitted:(NSTextField *)sender {
    NSString *command = [NSString stringWithFormat:@"set-text|%@|%@", sender.identifier, sender.stringValue];
    [self dispatchAndRefresh:command];
}

- (void)commandButtonPressed:(NSButton *)sender {
    [self dispatchAndRefresh:sender.identifier ?: @""];
}

- (void)dispatchAndRefresh:(NSString *)command {
    self.dispatchCommand(self.bridgeContext, command.UTF8String);
    [self refreshUI];
}

- (void)sliderInteractionDidEnd:(NSSlider *)sender {
    (void)sender;
    [self refreshUI];
}

- (void)updateScalarValueLabelForSlider:(NSSlider *)slider {
    NSView *parent = slider.superview;
    if (![parent isKindOfClass:[NSStackView class]]) {
        return;
    }

    NSStackView *row = (NSStackView *)parent;
    if (row.arrangedSubviews.count < 3) {
        return;
    }

    NSTextField *valueView = (NSTextField *)row.arrangedSubviews[2];
    if (![valueView isKindOfClass:[NSTextField class]]) {
        return;
    }

    valueView.stringValue = [self formattedScalarValueForSlider:slider];
}

- (NSString *)formattedScalarValueForSlider:(NSSlider *)slider {
    NSString *fieldId = slider.identifier ?: @"";
    if ([fieldId isEqualToString:@"param:background_hue"] || [fieldId isEqualToString:@"param:accent_hue"]) {
        return [NSString stringWithFormat:@"%6.1f", slider.doubleValue];
    }
    return [NSString stringWithFormat:@"%6.0f%%", slider.doubleValue * 100.0];
}

- (void)applySnapshot:(NSDictionary *)snapshot
      rebuildControls:(BOOL)rebuildControls
     preservingSliderId:(NSString *)preservedSliderId {
    self.snapshot = snapshot;
    self.tokens = snapshot[@"tokens"] ?: @[];
    self.window.title = snapshot[@"window_title"] ?: @"Theme Generator";
    self.statusLabel.stringValue = snapshot[@"status"] ?: @"";

    [self.tokenTable reloadData];
    NSInteger selectedToken = [self selectedTokenIndex];
    self.suppressTokenSelection = YES;
    if (selectedToken >= 0 && selectedToken < (NSInteger)self.tokens.count) {
        [self.tokenTable selectRowIndexes:[NSIndexSet indexSetWithIndex:(NSUInteger)selectedToken]
                     byExtendingSelection:NO];
        [self.tokenTable scrollRowToVisible:selectedToken];
    }
    self.suppressTokenSelection = NO;

    NSArray<NSDictionary *> *params = snapshot[@"params"] ?: @[];
    NSDictionary *inspector = snapshot[@"inspector"] ?: @{};

    if (rebuildControls) {
        [self rebuildParams:params];
        [self rebuildInspector:inspector];
    } else {
        [self syncScalarRowsInStack:self.paramsStack
                         withFields:params
                 preservingSliderId:preservedSliderId];
        [self syncInspector:inspector preservingSliderId:preservedSliderId];
    }

    [self rebuildPalette:snapshot[@"palette"] ?: @[]];
    [self rebuildPreview:snapshot];
}

- (void)syncInspector:(NSDictionary *)inspector preservingSliderId:(NSString *)preservedSliderId {
    NSString *tokenLabel = inspector[@"token_label"] ?: @"Token";
    NSString *tokenHex = inspector[@"token_color_hex"] ?: @"#000000";
    NSString *summary = inspector[@"rule_summary"] ?: @"";

    self.inspectorTitleLabel.stringValue = tokenLabel;
    [self configureSwatchView:self.inspectorSwatch withHex:tokenHex];
    self.inspectorSummaryLabel.stringValue = summary;

    NSArray<NSDictionary *> *fields = inspector[@"fields"] ?: @[];
    if (![self syncFieldRowsInStack:self.inspectorFieldsStack
                         withFields:fields
                 preservingSliderId:preservedSliderId]) {
        [self rebuildInspector:inspector];
    }
}

- (void)syncScalarRowsInStack:(NSStackView *)stack
                   withFields:(NSArray<NSDictionary *> *)fields
           preservingSliderId:(NSString *)preservedSliderId {
    if (stack.arrangedSubviews.count != fields.count) {
        [self rebuildParams:fields];
        return;
    }

    for (NSUInteger index = 0; index < fields.count; index += 1) {
        NSDictionary *field = fields[index];
        NSView *row = stack.arrangedSubviews[index];
        if (![row isKindOfClass:[NSStackView class]]) {
            [self rebuildParams:fields];
            return;
        }

        if (![self syncScalarRow:(NSStackView *)row
                       withField:field
               preservingSliderId:preservedSliderId]) {
            [self rebuildParams:fields];
            return;
        }
    }
}

- (BOOL)syncFieldRowsInStack:(NSStackView *)stack
                  withFields:(NSArray<NSDictionary *> *)fields
          preservingSliderId:(NSString *)preservedSliderId {
    if (stack.arrangedSubviews.count != fields.count) {
        return NO;
    }

    for (NSUInteger index = 0; index < fields.count; index += 1) {
        NSDictionary *field = fields[index];
        NSString *kind = field[@"kind"] ?: @"";
        NSView *row = stack.arrangedSubviews[index];
        if (![row isKindOfClass:[NSStackView class]]) {
            return NO;
        }

        if ([kind isEqualToString:@"scalar"]) {
            if (![self syncScalarRow:(NSStackView *)row
                           withField:field
                   preservingSliderId:preservedSliderId]) {
                return NO;
            }
        } else if ([kind isEqualToString:@"choice"]) {
            if (![self syncChoiceRow:(NSStackView *)row withField:field]) {
                return NO;
            }
        } else if ([kind isEqualToString:@"color"]) {
            if (![self syncColorRow:(NSStackView *)row withField:field]) {
                return NO;
            }
        } else {
            return NO;
        }
    }

    return YES;
}

- (BOOL)syncScalarRow:(NSStackView *)row
            withField:(NSDictionary *)field
    preservingSliderId:(NSString *)preservedSliderId {
    NSString *fieldId = field[@"id"] ?: @"";
    if (row.arrangedSubviews.count < 3) {
        return NO;
    }

    NSTextField *labelView = (NSTextField *)row.arrangedSubviews[0];
    NSSlider *slider = (NSSlider *)row.arrangedSubviews[1];
    NSTextField *valueView = (NSTextField *)row.arrangedSubviews[2];
    if (![slider isKindOfClass:[NSSlider class]] || ![valueView isKindOfClass:[NSTextField class]]) {
        return NO;
    }

    labelView.stringValue = field[@"label"] ?: @"";
    slider.minValue = [field[@"min"] doubleValue];
    slider.maxValue = [field[@"max"] doubleValue];
    if (preservedSliderId == nil || ![slider.identifier isEqualToString:preservedSliderId]) {
        slider.doubleValue = [field[@"current"] doubleValue];
    }
    slider.identifier = fieldId;
    valueView.stringValue = field[@"value_text"] ?: @"";
    return YES;
}

- (BOOL)syncChoiceRow:(NSStackView *)row withField:(NSDictionary *)field {
    if (row.arrangedSubviews.count < 2) {
        return NO;
    }

    NSTextField *labelView = (NSTextField *)row.arrangedSubviews[0];
    NSPopUpButton *popup = (NSPopUpButton *)row.arrangedSubviews[1];
    if (![popup isKindOfClass:[NSPopUpButton class]]) {
        return NO;
    }

    NSString *selectedKey = field[@"selected_key"] ?: @"";
    labelView.stringValue = field[@"label"] ?: @"";
    popup.identifier = field[@"id"] ?: @"";

    NSArray<NSDictionary *> *options = field[@"options"] ?: @[];
    if (popup.numberOfItems != (NSInteger)options.count) {
        return NO;
    }

    NSInteger selectedIndex = 0;
    for (NSUInteger index = 0; index < options.count; index += 1) {
        NSDictionary *option = options[index];
        NSString *title = option[@"label"] ?: @"";
        NSString *key = option[@"key"] ?: @"";
        NSMenuItem *item = [popup itemAtIndex:(NSInteger)index];
        item.title = title;
        item.representedObject = key;
        if ([key isEqualToString:selectedKey]) {
            selectedIndex = (NSInteger)index;
        }
    }
    [popup selectItemAtIndex:selectedIndex];
    return YES;
}

- (BOOL)syncColorRow:(NSStackView *)row withField:(NSDictionary *)field {
    if (row.arrangedSubviews.count < 3) {
        return NO;
    }

    NSTextField *labelView = (NSTextField *)row.arrangedSubviews[0];
    NSView *swatch = row.arrangedSubviews[1];
    NSTextField *textField = (NSTextField *)row.arrangedSubviews[2];
    if (![textField isKindOfClass:[NSTextField class]]) {
        return NO;
    }

    labelView.stringValue = field[@"label"] ?: @"";
    [self configureSwatchView:swatch withHex:field[@"color_hex"] ?: @"#000000"];
    textField.identifier = field[@"id"] ?: @"";
    if (textField.currentEditor == nil) {
        textField.stringValue = field[@"value_text"] ?: @"";
    }
    return YES;
}

- (NSInteger)numberOfRowsInTableView:(NSTableView *)tableView {
    (void)tableView;
    return (NSInteger)self.tokens.count;
}

- (NSView *)tableView:(NSTableView *)tableView
   viewForTableColumn:(NSTableColumn *)tableColumn
                  row:(NSInteger)row {
    (void)tableView;
    (void)tableColumn;

    NSTableCellView *cell = [tableView makeViewWithIdentifier:@"TokenCell" owner:self];
    if (cell == nil) {
        cell = [[NSTableCellView alloc] initWithFrame:NSMakeRect(0, 0, 180, 28)];
        cell.identifier = @"TokenCell";

        NSStackView *stack = [[NSStackView alloc] init];
        stack.orientation = NSUserInterfaceLayoutOrientationHorizontal;
        stack.alignment = NSLayoutAttributeCenterY;
        stack.spacing = 10.0;
        stack.translatesAutoresizingMaskIntoConstraints = NO;

        NSView *swatch = [self swatchViewWithHex:@"#000000" size:NSMakeSize(10.0, 10.0) radius:5.0];
        swatch.identifier = @"TokenSwatch";
        [stack addArrangedSubview:swatch];

        NSTextField *label = [NSTextField labelWithString:@""];
        label.identifier = @"TokenLabel";
        label.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
        [stack addArrangedSubview:label];

        [cell addSubview:stack];
        [NSLayoutConstraint activateConstraints:@[
            [stack.leadingAnchor constraintEqualToAnchor:cell.leadingAnchor constant:10.0],
            [stack.trailingAnchor constraintLessThanOrEqualToAnchor:cell.trailingAnchor constant:-8.0],
            [stack.centerYAnchor constraintEqualToAnchor:cell.centerYAnchor],
        ]];
    }

    NSDictionary *token = self.tokens[(NSUInteger)row];
    NSStackView *stack = (NSStackView *)cell.subviews.firstObject;
    NSView *swatch = stack.arrangedSubviews.count > 0 ? stack.arrangedSubviews[0] : nil;
    NSTextField *label = stack.arrangedSubviews.count > 1 ? (NSTextField *)stack.arrangedSubviews[1] : nil;
    [self configureSwatchView:swatch withHex:token[@"color_hex"] ?: @"#000000"];
    label.stringValue = token[@"label"] ?: @"";
    return cell;
}

- (void)tableViewSelectionDidChange:(NSNotification *)notification {
    (void)notification;
    if (self.suppressTokenSelection) {
        return;
    }

    NSInteger row = self.tokenTable.selectedRow;
    if (row >= 0) {
        NSString *command = [NSString stringWithFormat:@"select-token|%ld", (long)row];
        [self dispatchAndRefresh:command];
    }
}

@end

int theme_gui_host_run(void *context,
                       ThemeCopySnapshotFn copySnapshot,
                       ThemeDispatchCommandFn dispatchCommand,
                       ThemeFreeStringFn freeString) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];

        ThemeGuiController *delegate =
            [[ThemeGuiController alloc] initWithContext:context
                                           copySnapshot:copySnapshot
                                        dispatchCommand:dispatchCommand
                                             freeString:freeString];
        [NSApp setDelegate:delegate];
        [NSApp run];
    }

    return 0;
}
