//! Component copy in English and Chinese; `text` reads it when the app's catalogs lack the key.

/// English, the canonical set; zh-CN holds the same keys, a test keeps parity.
const EN: &[(&str, &str)] = &[
    ("palette.command.placeholder", "Type a command"),
    ("palette.command.empty", "No matching commands"),
    ("palette.recent", "Recent"),
    ("palette.open.placeholder", "Go to file"),
    ("palette.open.idle", "Type part of a file name"),
    ("palette.open.empty", "No matching files"),
    ("palette.switch.placeholder", "Switch to"),
    ("palette.switch.empty", "Nothing open by that name"),
    ("palette.search.placeholder", "Search everything"),
    ("palette.search.idle", "Type to search"),
    ("palette.search.empty", "Nothing found for “{query}”"),
    ("palette.symbol.placeholder", "Go to a symbol"),
    ("palette.symbol.empty", "No matching symbols"),
    ("palette.project.placeholder", "Switch to a project"),
    ("palette.project.empty", "No matching projects"),
    ("palette.project.pin", "Pin"),
    ("palette.project.unpin", "Unpin"),
    ("palette.project.remove", "Remove from recent"),
    (
        "palette.branch.placeholder",
        "Switch to a branch, or name a new one",
    ),
    ("palette.branch.local", "Local"),
    ("palette.branch.remote", "Remote"),
    ("palette.branch.create", "Create branch “{query}”"),
    ("palette.branch.empty", "No branch fits"),
    ("palette.branch.switch", "Switch to it"),
    ("palette.branch.delete", "Delete"),
    ("palette.history.placeholder", "Search past commands"),
    ("palette.history.empty", "No past command fits"),
    ("dialog.cancel", "Cancel"),
    ("dialog.close", "Close"),
    ("state.unavailable", "Unavailable"),
    ("templates.find", "Find a template"),
    ("templates.all", "All"),
    ("templates.list", "Templates"),
    ("templates.use", "Use template"),
    ("templates.none", "No template matches"),
    ("comments.reply", "Reply"),
    ("comments.resolve", "Resolve"),
    ("comments.reopen", "Reopen"),
    ("comments.open", "Open {n}"),
    ("comments.resolved", "Resolved {n}"),
    ("comments.none_open", "No open comments"),
    ("comments.none_resolved", "Nothing resolved yet"),
    ("comments.none", "No comments"),
    ("time.now", "just now"),
    ("time.ago", "{span} ago"),
    ("time.in", "in {span}"),
    ("time.minute.one", "{n} minute"),
    ("time.minute.other", "{n} minutes"),
    ("time.hour.one", "{n} hour"),
    ("time.hour.other", "{n} hours"),
    ("time.day.one", "{n} day"),
    ("time.day.other", "{n} days"),
    ("time.month.one", "{n} month"),
    ("time.month.other", "{n} months"),
    ("time.year.one", "{n} year"),
    ("time.year.other", "{n} years"),
];

const ZH_CN: &[(&str, &str)] = &[
    ("palette.command.placeholder", "输入命令"),
    ("palette.command.empty", "没有匹配的命令"),
    ("palette.recent", "最近"),
    ("palette.open.placeholder", "转到文件"),
    ("palette.open.idle", "输入文件名的一部分"),
    ("palette.open.empty", "没有匹配的文件"),
    ("palette.switch.placeholder", "切换到"),
    ("palette.switch.empty", "没有以此名字打开的"),
    ("palette.search.placeholder", "搜索全部"),
    ("palette.search.idle", "输入以搜索"),
    ("palette.search.empty", "没有找到“{query}”"),
    ("palette.symbol.placeholder", "转到符号"),
    ("palette.symbol.empty", "没有匹配的符号"),
    ("palette.project.placeholder", "切换项目"),
    ("palette.project.empty", "没有匹配的项目"),
    ("palette.project.pin", "置顶"),
    ("palette.project.unpin", "取消置顶"),
    ("palette.project.remove", "从最近中移除"),
    ("palette.branch.placeholder", "切换分支，或输入新分支名"),
    ("palette.branch.local", "本地"),
    ("palette.branch.remote", "远程"),
    ("palette.branch.create", "创建分支“{query}”"),
    ("palette.branch.empty", "没有匹配的分支"),
    ("palette.branch.switch", "切换过去"),
    ("palette.branch.delete", "删除"),
    ("palette.history.placeholder", "搜索历史命令"),
    ("palette.history.empty", "没有匹配的历史命令"),
    ("dialog.cancel", "取消"),
    ("dialog.close", "关闭"),
    ("state.unavailable", "不可用"),
    ("templates.find", "查找模板"),
    ("templates.all", "全部"),
    ("templates.list", "模板"),
    ("templates.use", "使用模板"),
    ("templates.none", "没有匹配的模板"),
    ("comments.reply", "回复"),
    ("comments.resolve", "解决"),
    ("comments.reopen", "重新打开"),
    ("comments.open", "未解决 {n}"),
    ("comments.resolved", "已解决 {n}"),
    ("comments.none_open", "没有未解决的批注"),
    ("comments.none_resolved", "还没有已解决的批注"),
    ("comments.none", "没有批注"),
    ("time.now", "刚刚"),
    ("time.ago", "{span}前"),
    ("time.in", "{span}后"),
    ("time.minute.one", "{n} 分钟"),
    ("time.minute.other", "{n} 分钟"),
    ("time.hour.one", "{n} 小时"),
    ("time.hour.other", "{n} 小时"),
    ("time.day.one", "{n} 天"),
    ("time.day.other", "{n} 天"),
    ("time.month.one", "{n} 个月"),
    ("time.month.other", "{n} 个月"),
    ("time.year.one", "{n} 年"),
    ("time.year.other", "{n} 年"),
];

/// Ely's own message for `key` in `locale`, English for a locale Ely does not ship; fails on an unknown key.
pub(crate) fn builtin(locale: &str, key: &str) -> &'static str {
    let table = if locale == "zh-CN" { ZH_CN } else { EN };
    table
        .iter()
        .find_map(|(k, v)| (*k == key).then_some(*v))
        .or_else(|| EN.iter().find_map(|(k, v)| (*k == key).then_some(*v)))
        .unwrap_or_else(|| panic!("no builtin message {key:?}"))
}

#[cfg(test)]
mod tests {
    use super::{EN, ZH_CN};

    #[test]
    fn zh_cn_holds_every_english_key() {
        let zh: std::collections::HashSet<_> = ZH_CN.iter().map(|(key, _)| key).collect();
        assert_eq!(EN.len(), ZH_CN.len());
        assert!(EN.iter().all(|(key, _)| zh.contains(key)));
    }
}
