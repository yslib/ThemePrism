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

@interface ThemePaneSplitView : NSSplitView

@end

static NSColor *ThemeColorFromHex(NSString *hex) {
    NSString *value = [[hex stringByTrimmingCharactersInSet:[NSCharacterSet whitespaceAndNewlineCharacterSet]]
        stringByReplacingOccurrencesOfString:@"#" withString:@""];
    if (value.length != 6 && value.length != 8) {
        return [NSColor clearColor];
    }

    unsigned int raw = 0;
    if (![[NSScanner scannerWithString:value] scanHexInt:&raw]) {
        return [NSColor clearColor];
    }

    CGFloat red = 0.0;
    CGFloat green = 0.0;
    CGFloat blue = 0.0;
    CGFloat alpha = 1.0;

    if (value.length == 8) {
        red = ((raw >> 24) & 0xFF) / 255.0;
        green = ((raw >> 16) & 0xFF) / 255.0;
        blue = ((raw >> 8) & 0xFF) / 255.0;
        alpha = (raw & 0xFF) / 255.0;
    } else {
        red = ((raw >> 16) & 0xFF) / 255.0;
        green = ((raw >> 8) & 0xFF) / 255.0;
        blue = (raw & 0xFF) / 255.0;
    }

    return [NSColor colorWithSRGBRed:red green:green blue:blue alpha:alpha];
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
@property (nonatomic, strong) NSStackView *editorConfigStack;
@property (nonatomic, strong) NSWindow *configSheet;
@property (nonatomic, strong) NSStackView *configSheetStack;
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

@implementation ThemePaneSplitView

- (CGFloat)dividerThickness {
    return 10.0;
}

- (void)drawDividerInRect:(NSRect)rect {
    (void)rect;
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
    self.window.opaque = YES;
    self.window.backgroundColor = NSColor.windowBackgroundColor;
    self.window.title = @"Theme Generator";
    self.window.titlebarAppearsTransparent = NO;
    self.window.toolbarStyle = NSWindowToolbarStyleUnified;
    self.window.minSize = NSMakeSize(1180, 760);

    NSView *contentView = self.window.contentView;
    NSVisualEffectView *backgroundView = [[NSVisualEffectView alloc] init];
    backgroundView.translatesAutoresizingMaskIntoConstraints = NO;
    backgroundView.material = NSVisualEffectMaterialUnderWindowBackground;
    backgroundView.blendingMode = NSVisualEffectBlendingModeBehindWindow;
    backgroundView.state = NSVisualEffectStateActive;
    [contentView addSubview:backgroundView];

    ThemePaneSplitView *splitView = [[ThemePaneSplitView alloc] init];
    splitView.translatesAutoresizingMaskIntoConstraints = NO;
    splitView.vertical = YES;
    splitView.dividerStyle = NSSplitViewDividerStyleThin;
    splitView.wantsLayer = YES;

    NSView *leftPane = [self paneContainerWithContentView:[self buildSidebar]];
    NSView *centerPane = [self paneContainerWithContentView:[self buildCenterPane]];
    NSView *rightPane = [self paneContainerWithContentView:[self buildInspectorPane]];

    [splitView addArrangedSubview:leftPane];
    [splitView addArrangedSubview:centerPane];
    [splitView addArrangedSubview:rightPane];

    [leftPane.widthAnchor constraintEqualToConstant:246.0].active = YES;
    [rightPane.widthAnchor constraintEqualToConstant:376.0].active = YES;

    NSView *mainPanel = [[NSView alloc] init];
    mainPanel.translatesAutoresizingMaskIntoConstraints = NO;
    [contentView addSubview:mainPanel];
    [mainPanel addSubview:splitView];

    NSVisualEffectView *statusBar = [[NSVisualEffectView alloc] init];
    statusBar.translatesAutoresizingMaskIntoConstraints = NO;
    statusBar.material = NSVisualEffectMaterialSidebar;
    statusBar.blendingMode = NSVisualEffectBlendingModeBehindWindow;
    statusBar.state = NSVisualEffectStateActive;
    statusBar.wantsLayer = YES;
    statusBar.layer.borderWidth = 0.0;
    statusBar.layer.backgroundColor =
        [[[NSColor windowBackgroundColor] colorWithAlphaComponent:0.03] CGColor];

    self.statusLabel = [NSTextField labelWithString:@"Ready"];
    self.statusLabel.translatesAutoresizingMaskIntoConstraints = NO;
    self.statusLabel.font = [NSFont systemFontOfSize:12.0 weight:NSFontWeightRegular];
    self.statusLabel.textColor = NSColor.secondaryLabelColor;
    [statusBar addSubview:self.statusLabel];

    [contentView addSubview:statusBar];

    [NSLayoutConstraint activateConstraints:@[
        [backgroundView.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [backgroundView.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [backgroundView.topAnchor constraintEqualToAnchor:contentView.topAnchor],
        [backgroundView.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],

        [mainPanel.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor constant:14.0],
        [mainPanel.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor constant:-14.0],
        [mainPanel.topAnchor constraintEqualToAnchor:contentView.topAnchor constant:14.0],
        [mainPanel.bottomAnchor constraintEqualToAnchor:statusBar.topAnchor constant:-10.0],

        [splitView.leadingAnchor constraintEqualToAnchor:mainPanel.leadingAnchor],
        [splitView.trailingAnchor constraintEqualToAnchor:mainPanel.trailingAnchor],
        [splitView.topAnchor constraintEqualToAnchor:mainPanel.topAnchor],
        [splitView.bottomAnchor constraintEqualToAnchor:mainPanel.bottomAnchor],

        [statusBar.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [statusBar.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [statusBar.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],
        [statusBar.heightAnchor constraintEqualToConstant:34.0],

        [self.statusLabel.leadingAnchor constraintEqualToAnchor:statusBar.leadingAnchor constant:16.0],
        [self.statusLabel.trailingAnchor constraintEqualToAnchor:statusBar.trailingAnchor constant:-16.0],
        [self.statusLabel.centerYAnchor constraintEqualToAnchor:statusBar.centerYAnchor],
    ]];
}

- (NSView *)paneContainerWithContentView:(NSView *)content {
    NSVisualEffectView *container = [[NSVisualEffectView alloc] init];
    container.translatesAutoresizingMaskIntoConstraints = NO;
    container.material = NSVisualEffectMaterialSidebar;
    container.blendingMode = NSVisualEffectBlendingModeWithinWindow;
    container.state = NSVisualEffectStateActive;
    container.wantsLayer = YES;
    container.layer.cornerRadius = 13.0;
    container.layer.masksToBounds = YES;
    container.layer.borderWidth = 1.0;
    container.layer.borderColor =
        [[NSColor.separatorColor colorWithAlphaComponent:0.18] CGColor];
    container.layer.backgroundColor =
        [[NSColor.controlBackgroundColor colorWithAlphaComponent:0.05] CGColor];

    content.translatesAutoresizingMaskIntoConstraints = NO;
    [container addSubview:content];

    [NSLayoutConstraint activateConstraints:@[
        [content.leadingAnchor constraintEqualToAnchor:container.leadingAnchor constant:1.0],
        [content.trailingAnchor constraintEqualToAnchor:container.trailingAnchor constant:-1.0],
        [content.topAnchor constraintEqualToAnchor:container.topAnchor constant:1.0],
        [content.bottomAnchor constraintEqualToAnchor:container.bottomAnchor constant:-1.0],
    ]];

    return container;
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

    [stack addArrangedSubview:[self sectionLabel:@"Editor Preferences"]];
    self.editorConfigStack = [self verticalSectionStack];
    [stack addArrangedSubview:self.editorConfigStack];

    [stack addArrangedSubview:[self sectionLabel:@"Actions"]];
    NSStackView *actions = [[NSStackView alloc] init];
    actions.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    actions.spacing = 10.0;
    actions.distribution = NSStackViewDistributionFillEqually;
    actions.translatesAutoresizingMaskIntoConstraints = NO;
    [actions addArrangedSubview:[self dialogButtonWithTitle:@"Config" action:@selector(openConfigSheet:)]];
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

- (void)rebuildEditorConfig:(NSDictionary *)editorConfig {
    [self clearStack:self.editorConfigStack];
    NSArray<NSDictionary *> *fields = editorConfig[@"fields"] ?: @[];
    for (NSDictionary *field in fields) {
        NSString *kind = field[@"kind"] ?: @"";
        if ([kind isEqualToString:@"text"]) {
            [self.editorConfigStack addArrangedSubview:[self configTextRowForField:field]];
        } else if ([kind isEqualToString:@"toggle"]) {
            [self.editorConfigStack addArrangedSubview:[self configToggleRowForField:field]];
        } else if ([kind isEqualToString:@"choice"]) {
            [self.editorConfigStack addArrangedSubview:[self configChoiceRowForField:field]];
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
    textField.placeholderString = @"#224466 or #224466CC";
    textField.font = [NSFont monospacedSystemFontOfSize:12.0 weight:NSFontWeightRegular];
    textField.target = self;
    textField.action = @selector(textCommitted:);
    textField.delegate = self;
    [textField.widthAnchor constraintGreaterThanOrEqualToConstant:160.0].active = YES;
    [row addArrangedSubview:textField];

    return row;
}

- (NSView *)configTextRowForField:(NSDictionary *)field {
    NSString *label = field[@"label"] ?: @"";
    NSString *fieldId = field[@"id"] ?: @"";
    NSString *value = field[@"value_text"] ?: @"";

    NSStackView *row = [[NSStackView alloc] init];
    row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    row.alignment = NSLayoutAttributeCenterY;
    row.spacing = 12.0;

    NSTextField *labelView = [NSTextField labelWithString:label];
    labelView.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
    [labelView.widthAnchor constraintEqualToConstant:130.0].active = YES;
    [row addArrangedSubview:labelView];

    NSTextField *textField = [[NSTextField alloc] init];
    textField.identifier = fieldId;
    textField.stringValue = value;
    textField.font = [NSFont monospacedSystemFontOfSize:12.0 weight:NSFontWeightRegular];
    textField.target = self;
    textField.action = @selector(configTextCommitted:);
    textField.delegate = self;
    [textField.widthAnchor constraintGreaterThanOrEqualToConstant:180.0].active = YES;
    [row addArrangedSubview:textField];

    return row;
}

- (NSView *)configToggleRowForField:(NSDictionary *)field {
    NSString *label = field[@"label"] ?: @"";
    NSString *fieldId = field[@"id"] ?: @"";
    NSString *value = field[@"value_text"] ?: @"";

    NSStackView *row = [[NSStackView alloc] init];
    row.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    row.alignment = NSLayoutAttributeCenterY;
    row.spacing = 12.0;

    NSTextField *labelView = [NSTextField labelWithString:label];
    labelView.font = [NSFont systemFontOfSize:12.5 weight:NSFontWeightMedium];
    [labelView.widthAnchor constraintEqualToConstant:130.0].active = YES;
    [row addArrangedSubview:labelView];

    NSButton *toggle = [NSButton checkboxWithTitle:value target:self action:@selector(configToggleChanged:)];
    toggle.identifier = fieldId;
    toggle.state = [field[@"enabled"] boolValue] ? NSControlStateValueOn : NSControlStateValueOff;
    [row addArrangedSubview:toggle];

    return row;
}

- (NSView *)configChoiceRowForField:(NSDictionary *)field {
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
    popup.action = @selector(configChoiceChanged:);
    [popup.widthAnchor constraintGreaterThanOrEqualToConstant:180.0].active = YES;

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

- (void)ensureConfigSheet {
    if (self.configSheet != nil) {
        return;
    }

    self.configSheet = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 620, 560)
                                                   styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable)
                                                     backing:NSBackingStoreBuffered
                                                       defer:NO];
    self.configSheet.title = @"Configuration";
    self.configSheet.releasedWhenClosed = NO;

    NSView *contentView = self.configSheet.contentView;
    NSStackView *root = [[NSStackView alloc] init];
    root.orientation = NSUserInterfaceLayoutOrientationVertical;
    root.spacing = 14.0;
    root.edgeInsets = NSEdgeInsetsMake(18.0, 18.0, 18.0, 18.0);
    root.translatesAutoresizingMaskIntoConstraints = NO;

    self.configSheetStack = [self verticalSectionStack];
    self.configSheetStack.spacing = 14.0;

    NSScrollView *scrollView = [[NSScrollView alloc] init];
    scrollView.translatesAutoresizingMaskIntoConstraints = NO;
    scrollView.hasVerticalScroller = YES;
    scrollView.drawsBackground = NO;
    scrollView.documentView = self.configSheetStack;
    [scrollView.heightAnchor constraintEqualToConstant:430.0].active = YES;

    NSStackView *actions = [[NSStackView alloc] init];
    actions.orientation = NSUserInterfaceLayoutOrientationHorizontal;
    actions.alignment = NSLayoutAttributeCenterY;
    actions.spacing = 10.0;
    actions.distribution = NSStackViewDistributionGravityAreas;
    actions.translatesAutoresizingMaskIntoConstraints = NO;

    [actions addArrangedSubview:[NSTextField labelWithString:@"Project settings and editor preferences"]];
    [actions addArrangedSubview:[self dialogButtonWithTitle:@"Done" action:@selector(closeConfigSheet:)]];

    [root addArrangedSubview:scrollView];
    [root addArrangedSubview:actions];
    [contentView addSubview:root];

    [NSLayoutConstraint activateConstraints:@[
        [root.leadingAnchor constraintEqualToAnchor:contentView.leadingAnchor],
        [root.trailingAnchor constraintEqualToAnchor:contentView.trailingAnchor],
        [root.topAnchor constraintEqualToAnchor:contentView.topAnchor],
        [root.bottomAnchor constraintEqualToAnchor:contentView.bottomAnchor],
    ]];
}

- (void)openConfigSheet:(id)sender {
    (void)sender;
    [self ensureConfigSheet];
    [self rebuildConfigSheetFromSnapshot:self.snapshot[@"config_sheet"] ?: @{}];
    if (self.window.attachedSheet != self.configSheet) {
        [self.window beginSheet:self.configSheet completionHandler:nil];
    }
}

- (void)closeConfigSheet:(id)sender {
    (void)sender;
    if (self.window.attachedSheet == self.configSheet) {
        [self.window endSheet:self.configSheet];
    }
    [self.configSheet orderOut:nil];
}

- (void)rebuildConfigSheetFromSnapshot:(NSDictionary *)sheet {
    if (self.configSheetStack == nil) {
        return;
    }

    [self clearStack:self.configSheetStack];

    [self.configSheetStack addArrangedSubview:[self sectionLabel:@"Project"]];
    NSDictionary *projectName = sheet[@"project_name"] ?: @{};
    [self.configSheetStack addArrangedSubview:[self configTextRowForField:projectName]];

    [self.configSheetStack addArrangedSubview:[self sectionLabel:@"Export Targets"]];
    NSArray<NSDictionary *> *targets = sheet[@"export_targets"] ?: @[];
    for (NSDictionary *target in targets) {
        [self.configSheetStack addArrangedSubview:[self exportTargetBlockForItem:target]];
    }

    [self.configSheetStack addArrangedSubview:[self sectionLabel:@"Editor Preferences"]];
    NSArray<NSDictionary *> *editorFields = sheet[@"editor_fields"] ?: @[];
    for (NSDictionary *field in editorFields) {
        NSString *kind = field[@"kind"] ?: @"";
        if ([kind isEqualToString:@"text"]) {
            [self.configSheetStack addArrangedSubview:[self configTextRowForField:field]];
        } else if ([kind isEqualToString:@"toggle"]) {
            [self.configSheetStack addArrangedSubview:[self configToggleRowForField:field]];
        } else if ([kind isEqualToString:@"choice"]) {
            [self.configSheetStack addArrangedSubview:[self configChoiceRowForField:field]];
        }
    }
}

- (NSView *)exportTargetBlockForItem:(NSDictionary *)item {
    NSInteger index = [item[@"index"] integerValue];
    NSString *label = item[@"label"] ?: @"Export";
    NSString *outputPath = item[@"output_path"] ?: @"";
    NSString *templatePath = item[@"template_path"];

    NSStackView *stack = [self verticalSectionStack];
    stack.spacing = 8.0;
    stack.edgeInsets = NSEdgeInsetsMake(8.0, 0.0, 6.0, 0.0);

    NSButton *toggle = [NSButton checkboxWithTitle:label target:self action:@selector(configToggleChanged:)];
    toggle.identifier = [NSString stringWithFormat:@"export_enabled:%ld", (long)index];
    toggle.state = [item[@"enabled"] boolValue] ? NSControlStateValueOn : NSControlStateValueOff;
    [stack addArrangedSubview:toggle];

    NSDictionary *outputField = @{
        @"id": [NSString stringWithFormat:@"export_output:%ld", (long)index],
        @"label": @"Output",
        @"value_text": outputPath,
    };
    [stack addArrangedSubview:[self configTextRowForField:outputField]];

    if ([templatePath isKindOfClass:[NSString class]]) {
        NSDictionary *templateField = @{
            @"id": [NSString stringWithFormat:@"export_template:%ld", (long)index],
            @"label": @"Template",
            @"value_text": templatePath,
        };
        [stack addArrangedSubview:[self configTextRowForField:templateField]];
    }

    return stack;
}

- (NSButton *)actionButtonWithTitle:(NSString *)title command:(NSString *)command {
    NSButton *button = [NSButton buttonWithTitle:title target:self action:@selector(commandButtonPressed:)];
    button.identifier = command;
    button.bezelStyle = NSBezelStyleRounded;
    return button;
}

- (NSButton *)dialogButtonWithTitle:(NSString *)title action:(SEL)action {
    NSButton *button = [NSButton buttonWithTitle:title target:self action:action];
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

    NSDictionary *snapshot = [self fetchSnapshot];
    if (snapshot == nil) {
        return;
    }

    [self applySnapshot:snapshot rebuildControls:NO preservingSliderId:sender.identifier];
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

- (void)configTextCommitted:(NSTextField *)sender {
    NSString *identifier = sender.identifier ?: @"";
    NSString *command = nil;

    if ([identifier isEqualToString:@"project_name"]) {
        command = [NSString stringWithFormat:@"set-project-name|%@", sender.stringValue];
    } else if ([identifier isEqualToString:@"project_path"]) {
        command = [NSString stringWithFormat:@"set-editor-text|%@|%@", identifier, sender.stringValue];
    } else if ([identifier hasPrefix:@"export_output:"]) {
        NSString *index = [identifier componentsSeparatedByString:@":"].lastObject ?: @"0";
        command = [NSString stringWithFormat:@"set-export-output|%@|%@", index, sender.stringValue];
    } else if ([identifier hasPrefix:@"export_template:"]) {
        NSString *index = [identifier componentsSeparatedByString:@":"].lastObject ?: @"0";
        command = [NSString stringWithFormat:@"set-export-template|%@|%@", index, sender.stringValue];
    }

    if (command == nil) {
        return;
    }

    [self dispatchAndRefresh:command];
}

- (void)configToggleChanged:(NSButton *)sender {
    NSString *identifier = sender.identifier ?: @"";
    NSString *value = sender.state == NSControlStateValueOn ? @"true" : @"false";
    NSString *command = nil;

    if ([identifier hasPrefix:@"export_enabled:"]) {
        NSString *index = [identifier componentsSeparatedByString:@":"].lastObject ?: @"0";
        command = [NSString stringWithFormat:@"set-export-enabled|%@|%@", index, value];
    } else {
        command = [NSString stringWithFormat:@"set-editor-toggle|%@|%@", identifier, value];
    }

    [self dispatchAndRefresh:command];
}

- (void)configChoiceChanged:(NSPopUpButton *)sender {
    NSString *value = sender.selectedItem.representedObject ?: @"";
    NSString *identifier = sender.identifier ?: @"";
    NSString *command = [NSString stringWithFormat:@"set-editor-choice|%@|%@", identifier, value];
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
    NSDictionary *configSheet = snapshot[@"config_sheet"] ?: @{};
    NSDictionary *editorConfig = snapshot[@"editor_config"] ?: @{};
    NSDictionary *inspector = snapshot[@"inspector"] ?: @{};

    if (rebuildControls) {
        [self rebuildParams:params];
        [self rebuildEditorConfig:editorConfig];
        [self rebuildInspector:inspector];
    } else {
        [self syncScalarRowsInStack:self.paramsStack
                         withFields:params
                 preservingSliderId:preservedSliderId];
        [self syncEditorConfig:editorConfig];
        [self syncInspector:inspector preservingSliderId:preservedSliderId];
    }

    [self rebuildPalette:snapshot[@"palette"] ?: @[]];
    [self rebuildPreview:snapshot];

    if (self.window.attachedSheet == self.configSheet) {
        [self rebuildConfigSheetFromSnapshot:configSheet];
    }
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

- (void)syncEditorConfig:(NSDictionary *)editorConfig {
    NSArray<NSDictionary *> *fields = editorConfig[@"fields"] ?: @[];
    if (![self syncConfigRowsInStack:self.editorConfigStack withFields:fields]) {
        [self rebuildEditorConfig:editorConfig];
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

- (BOOL)syncConfigRowsInStack:(NSStackView *)stack withFields:(NSArray<NSDictionary *> *)fields {
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

        if ([kind isEqualToString:@"text"]) {
            if (![self syncConfigTextRow:(NSStackView *)row withField:field]) {
                return NO;
            }
        } else if ([kind isEqualToString:@"toggle"]) {
            if (![self syncConfigToggleRow:(NSStackView *)row withField:field]) {
                return NO;
            }
        } else if ([kind isEqualToString:@"choice"]) {
            if (![self syncConfigChoiceRow:(NSStackView *)row withField:field]) {
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

- (BOOL)syncConfigTextRow:(NSStackView *)row withField:(NSDictionary *)field {
    if (row.arrangedSubviews.count < 2) {
        return NO;
    }

    NSTextField *labelView = (NSTextField *)row.arrangedSubviews[0];
    NSTextField *textField = (NSTextField *)row.arrangedSubviews[1];
    if (![textField isKindOfClass:[NSTextField class]]) {
        return NO;
    }

    labelView.stringValue = field[@"label"] ?: @"";
    textField.identifier = field[@"id"] ?: @"";
    if (textField.currentEditor == nil) {
        textField.stringValue = field[@"value_text"] ?: @"";
    }
    return YES;
}

- (BOOL)syncConfigToggleRow:(NSStackView *)row withField:(NSDictionary *)field {
    if (row.arrangedSubviews.count < 2) {
        return NO;
    }

    NSTextField *labelView = (NSTextField *)row.arrangedSubviews[0];
    NSButton *toggle = (NSButton *)row.arrangedSubviews[1];
    if (![toggle isKindOfClass:[NSButton class]]) {
        return NO;
    }

    labelView.stringValue = field[@"label"] ?: @"";
    toggle.identifier = field[@"id"] ?: @"";
    toggle.title = field[@"value_text"] ?: @"";
    toggle.state = [field[@"enabled"] boolValue] ? NSControlStateValueOn : NSControlStateValueOff;
    return YES;
}

- (BOOL)syncConfigChoiceRow:(NSStackView *)row withField:(NSDictionary *)field {
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
