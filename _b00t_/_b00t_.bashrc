#
# Purpose: universal bash b00t-strap for environment & tooling
#   once run in an enviroment will attempt to validate & construct
#   bash shortcuts, menus, etc.
#


# usage:
#   source "./_b00t_.bashrc"
#   may also *eventually* run via commandline.

# https://misc.flogisoft.com/bash/tip_colors_and_formatting
# https://github.com/awesome-lists/awesome-bash

# set -o errexit    # Used to exit upon error, avoiding cascading errors
set -o nounset    # Exposes unset variables, strict mode.
trap "set +o nounset" EXIT  # restore nounset at exit, even in crash!

# 🤔 trial:
umask 000


# mark variables which are modified or created for export (all BASH variables become export)
# NOTE: https://askubuntu.com/questions/1001653/why-am-i-getting-parse-usage-error-on-function-invocation-in-bash
#set -a
#trap 'set +a' EXIT



## 小路 \\
## Xiǎolù :: Path or Directory
# THINGS YOU CAN EDIT:
_B00T_Path="$HOME/.dotfiles/_b00t_"
if [ -d "$HOME/.dotfiles/_b00t_" ] ; then
    _B00T_Path="$HOME/.dotfiles/_b00t_"
fi
export _B00T_Path
# export _B00T_C0NFIG_Path="$HOME/.b00t"

# NSFW warning
_b00t_INSPIRATION_FILE="$_B00T_Path/./r3src_资源/inspiration.json"
## 小路 //


## 记录 \\
## Jìlù :: Record (Log)
# 🤓 write to a log if you want using >>
# mostly, this is for future opentelemetry & storytime log
unset -f log_📢_记录
function log_📢_记录() {
    echo "$@"
}
export -f log_📢_记录
## 记录 //

## this will allow b00t to restart itself.
unset -f reb00t
function reb00t() {
    unset -f _b00t_init_🥾_开始
    log_📢_记录 "🥾 restarting b00t"
    source "$_B00T_Path/_b00t_.bashrc"
}



function b00t() {
    # this is a placeholder for b00t-cli
    # it will eventually be replaced by b00t-cli
    if command -v b00t-cli &> /dev/null; then
        # b00t-cli is installed, use it.
        b00t-cli "$@"
    else
        # b00t-cli is not installed, use the script.
        log_📢_记录 "🥾: b00t-cli not found, using script"
        _b00t_init_🥾_开始 "$@"
    fi
    return 0
}

function 🥾() {
    # alias/passthrough fo b00t
    b00t "$@"
}



## 获取版本 \\
## Huòqǔ bǎnběn :: Get Version
unset -f _b00t_get_version
function _b00t_get_version() {
    local repo="$1"
    local version=$(cd "$repo" && git tag -l | sort -V | tail -n 1)
    echo "$version"
}
export -f _b00t_get_version





## * * * * * \\
## pathAdd
unset pathAdd
function pathAdd() {
    if [ -d "$1" ] && [[ ":$PATH:" != *":$1:"* ]]; then
        PATH="${PATH:+"$PATH:"}$1"
    fi
}
# webi tools
pathAdd "$HOME/.local/bin"
pathAdd "$HOME/.yarn/bin"
## * * * * * //


if [ "/usr/bin/docker" ] ; then
    echo "🐳 has d0cker! loading docker extensions"
    source "$_B00T_Path/docker.🐳/_bashrc.sh"

    ## 😔 docker context?
    ## https://docs.docker.com/engine/context/working-with-contexts/
    # export DOCKER_CONTEXT=default
    # log_📢_记录 "🐳 CONTEXT: $DOCKER_CONTEXT"
    # docker context ls

fi

## * * * * * \\
## is_version_大于
## compared #.#.# version (but could detect other formats)
## usage: is_version_大于 $requiredver $currentver
unset -f is_v3rs10n_大于
function is_v3rs10n_大于()   # $appversion $requiredversion
{
    # echo "hello $1 $2"
    # printf '%s\n%s\n' "$2" "$1" | sort --check=quiet --version-sort
    local appver=$1     # i.e. 1.0.0
    local requiredver=$2 # i.e. 1.0.1
    if [ "$(printf '%s\n' "$requiredver" "$appver" | sort -V | head -n1)" = "$requiredver" ]; then
        # version is insufficient
        result="false"
    else
        # version is sufficient (大于 greater than equal)
        result="true"
    fi

    echo $result
    return 0
}
export -f is_v3rs10n_大于
## * * * * * * //





##
# does a bash/function exist?
# 🍰 https://stackoverflow.com/questions/85880/determine-if-a-function-exists-in-bash
# returns:
#     0 on yes/success (function defined/available)
#     1 for no (not available)
function has_fn_✅_存在() {
    local exists=$(LC_ALL=C type -t $1)
    # echo "exists: $exists"
    if [ -n "$exists" ] ; then
        # exists, not empty, success
        return 0
    fi
    # fail (function does not exist)
    return 1
    # result=[[ -n "$exists" ]] || 1
}
export -f has_fn_✅_存在





## All the logic below figures out who is calling & hot-reload
## bail earlier is better,
_b00t_exists=`type -t "_b00t_init_🥾_开始"`
_b00t_VERSION_was=0
if [ "$_b00t_exists" == "function" ] ; then
    # are we re-entry, i.e. reb00t
    export _b00t_VERSION_was="$_b00t_VERSION"
fi
# -------------- CONFIGURABLE SETTING -----------------
export _b00t_VERSION="1.0.16"
# -----------------------------------------------------

# syntax: current required
# echo "v3r: $_b00t_VERSION "
upgradeB00T=$(is_v3rs10n_大于 "$_b00t_VERSION_was" "$_b00t_VERSION")
# echo "upgradeB00T: $upgradeB00T"

# 🦨 need consent!
if [ "$upgradeB00T" ==  true ] && [ -n "$_b00t_VERSION_was" ] ; then
    # welcome! (clean environment)
    log_📢_记录 "🥾🧐 b00t version | now: $_b00t_VERSION"
elif [ "$upgradeB00T" ==  true ] ; then
    ## upgrade b00t in memory (this doesn't work awesome, but useful during dev)
    ## $ reb00t
    log_📢_记录 "🥾🧐 (re)b00t version | now: $_b00t_VERSION | was: $_b00t_VERSION_was | upgrade: $upgradeB00T"
    # TODO: consent
elif [ "$_b00t_exists" == "function" ] ; then
    # SILENT, don't reload unless _b00t_VERSION is newer
    # short circuit using rand0() function
    # log_📢_记录 "👻 short-circuit"
    set +o nounset
    return
    ## 🍒 short circuit!
fi

## Have FZF use fdfind "fd" by default
export PS_FORMAT="pid,ppid,user,pri,ni,vsz,rss,pcpu,pmem,tty,stat,args"
export FD_OPTIONS="--follow -exlude .git --exclude node_modules"

## OPINIONATED ALIASES

# ☁️ cloud -cli's
function az_cli () {
    # local args=("$@")
    docker run --rm -it -v $HOME/.azure:/root/.azure -v $(pwd):/root mcr.microsoft.com/azure-cli:latest az $@
}
alias az="az_cli"
alias aws='docker run --rm -it -v ~/.aws:/root/.aws -v $(pwd):/aws amazon/aws-cli'
alias gcp="docker run --rm -ti --name gcloud-config google/cloud-sdk gcloud "


alias ls='ls -F  --color=auto'
# pretty = ll
#-rw-rw-r-- 1 1000 1000  100 May  5 23:51 requirements.txt
#-rw-rw-r-- 1 1000 1000  144 May  5 20:01 requirements.层_b00t_.txt
#-rw-rw-r-- 1 1000 1000  221 Apr 25 20:27 requirements.层_c0re.txt
alias ll='ls -lh'

#4.0K src/
#4.0K requirements.层_test.txt
#4.0K requirements.层_c0re.txt
alias lt='ls --human-readable --size -1 -S --classify'

# test for b00t-cli before defining alias
if [ -x "$HOME/.cargo/bin/b00t-cli" ] ; then
    # b00t-cli is installed, use it.
    alias b00t='b00t-cli'
else
    # b00t-cli is not installed, use the script.
    log_📢_记录 "🥾: b00t-cli not found, using script"
fi

alias s=ssh
alias c=clear
alias cx='chmod +x'
alias myp='ps -fjH -u $USER'

#mcd() { mkdir -p $1; cd $1 }
#export -f mcd
#cdl() { cd $1; ls}
#export -f cdl


# 🐍 Python
## moving from pipx to rye
# if ! command -v pipx &> /dev/null; then
#     echo "pipx could not be found, installing..."
#     if [ -n "$(whereis register-python-argcomplete3)" ] ; then
#         eval "$(register-python-argcomplete3 pipx)"
#         # pipx run
#     fi
# fi
#


# bat - a pretty replacement for cat.
alias bat="batcat"

# bats - bash testing system (in a docker container)
# 🦨 need consent before running docker
# 🦨 this is not the proper way to run bats.
# alias bats='docker run -it -v bats/bats:latest'

# count the files in a directory or project
alias count='find . -type f | wc -l'
# copy verbose, see rsync. or use preferred backup command.
alias cpv='rsync -ah --info=progress2'

# cd to _b00t_ (or current repo)
alias cg='cd `git rev-parse --show-toplevel`'
alias ..='cd ..'
alias ...='cd ../../'
alias mkdir='mkdir -pv'

# time & date
alias path='echo -e ${PATH//:/\n}'
alias now='date +"%T"'
alias nowtime=now
alias nowdate='date +"%d-%m-%Y"'

# 🐙 git
alias gitstatus='git -C . status --porcelain | grep "^.\w"'

# 🐍 Python ve = create .venv, va = activate!
alias ve='python3 -m venv ./venv'
alias va='source ./venv/bin/activate'

# use fzf to find a file and open it in vs code
alias c='code $(fzf --height 40% --reverse)'

# fd is same-same like unix find, but alt-featureset
# for example - fd respects .gitignore (but output like find)
if [ -f "/usr/bin/fdfind" ] ; then
    alias fd="/usr/bin/fdfind"
fi

# handy for generating dumps, etc..
# $ script.sh >> foobar.`ymd`
alias yyyymmdd="date +'%Y%m%d'"
alias ymd="date +'%Y%m%d'"
alias ymd_hm="date +'%Y%m%d.%H%M'"
alias ymd_hms="date +'%Y%m%d.%H%M%S'"
##################




# order of magnitude
#function oom () {
#    # todo: detect an order of magnitude transition.
#}





## 进口 \\
## Kāishǐ :: Start
# init should be run by every program.
# this is mostly here for StoryTime and future hooks.
unset -f _b00t_init_🥾_开始
function _b00t_init_🥾_开始() {
    local args=("$@")
    local param=${args[0]}

    export _b00t_="$0"
    if [ $_b00t_ == "/c0de/_b00t_/_b00t_.bashrc" ] ; then
        log_📢_记录 ""
        log_📢_记录 "usage: source /c0de/_b00t_/_b00t_.bashrc"
        exit
    fi

    # if b00t-cli isn't installed in the path,   # test for cargo, if cargo is installed AND
    # ~/.dotfiles/b00t-cli exists, then compile and install it

    if command -v b00t-cli &> /dev/null ; then
        log_📢_记录 "🥾👍"
        return 0
    elif command -v cargo &> /dev/null && [ -d "$HOME/.dotfiles/b00t-cli" ] ; then
        log_📢_记录 "🥾🦀 found cargo, will build + install b00t-cli"
        (cd "$HOME/.dotfiles/b00t-cli" && cargo build --release)
        if [ $? -eq 0 ] ; then
            log_📢_记录 "🥾 b00t-cli compiled successfully"
            (cd "$HOME/.dotfiles/b00t-cli" && cargo install --path .)
            return 0
        else
            log_📢_记录 "🥾🦨 b00t-cli build failed"
        fi
    fi

    local PARENT_COMMAND_STR="👽env-notdetected"
    if [ $PPID -eq 0 ] ; then
        if [ "$container" == "docker" ] ; then
            PARENT_COMMAND_STR="🐳 d0ck3r!"

        else
            PARENT_COMMAND_STR="👽env-unknown"
        fi
    else
        # lookup parent application by process id.
        PARENT_COMMAND_STR=$(ps -o comm=$PPID)
    fi

    if [ "$PARENT_COMMAND_STR" == "bash" ] ; then
        # most common case can be summarized
        log_📢_记录 "🥾👵:🔨"
    else
        log_📢_记录 "🥾👵 from: $PARENT_COMMAND_STR"
    fi

    log_📢_记录 "🥾 -V: $_b00t_VERSION  init: $_b00t_"
    if [ -n "${@}" ] ; then
        log_📢_记录 "🥾 args: ${@}"
    fi
}
export -f _b00t_init_🥾_开始
#_b00t_init_🥾_开始

## 进口 //


# alpine container support
# https://github.com/ethereum/solidity/issues/875
# returns 0 for "true" (not alpine linux), non-zero for false (is alpine linux)
function iz_n0t_alpine_linux_🐧🌲() {
   return $(cat /etc/os-release | grep "NAME=" | grep -ic "Alpine")
}
if [ ! iz_n0t_alpine_linux ] ; then
    # gh issue
    echo "🥾🤮 🐧🌲 alpine linux not fully supported yet"
fi


# this is intended to catch & report errors
function barf_🤮 () {
    gh issue create
    # gh issue create --title $1
}



# # Webi, presently breaks alpine config!
# # https://github.com/elasticdotventures/webi-installers
# webi=$(whereis webi)
# if [ -z "$webi" ] ; then
#     curl https://webinstall.dev/webi | bash
#     # Should install to $HOME/.local/opt/<package>-<version> or $HOME/.local/bin
#     # Should install to $HOME/.local/opt/<package>-<version> or $HOME/.local/bin
#     # Should not need sudo (except perhaps for a one-time setcap, etc)
# fi


#
# Returns seconds until a relative time i.e. "today 4pm"
#
function secondsUntil () {
    # pass "today 5:25pm"
    # same "today 17:25"
    # .. "tomorrow"
    WHEN=$1
    echo $(( $(date +%s -d "$WHEN") - $( date +%s ) ))
}


## 加载 * * * * * *\\
## Jiāzài :: Load
unset -f bash_source_加载
function bash_source_加载() {
    local file=$1

    log_📢_记录 "."
    log_📢_记录 "."

    # Bash Shell Parameter Expansion:
    # 🤓 https://www.gnu.org/software/bash/manual/html_node/Shell-Parameter-Expansion.html
    # The ‘$’ character introduces parameter expansion, command substitution, or arithmetic expansion.
    # {} are optional, but protect enclosed variable
    # when {} are used, the matching ending brace is the first ‘}’ not escaped by a backslash or within a quoted string, and not within an embedded arithmetic expansion, command substitution, or parameter expansion.
    # 🍰 https://www.xaprb.com/media/2007/03/bash-parameter-expansion-cheatsheet.pdf

    function expand { for arg in "$@"; do [[ -f $arg ]] && echo $arg; done }

    if [ ! -x "$file" ] ; then
        # .bashrc file doesn't exist, so let's try to find it.
        # trythis="${trythis:-$file}"
        # trythis=$file
        # ${trythis:-$file}
        #
        log_📢_记录 "🧐 expand $file"
        file=$( expand $file )

        if [ -x "$file" ] ; then
            log_📢_记录 "🧐 using $file"
        else
            log_📢_记录 "😲 NOT EXECUTABLE $file"
        fi

    fi

    if [ ! -x "$file" ] ; then
        log_📢_记录 "🗄️🔃😲🍒 NOT EXECUTABLE: $file" && exit
    else
        log_📢_记录 "🗄️🔃😏  START: $file"
        source "$file"
        if [ $? -gt 0 ] ; then
            echo "☹️🛑🛑🛑 ERROR: $file had runtime error! 🛑🛑🛑"
        fi
        log_📢_记录 "🗄️🔚😁 FINISH: $file"
    fi

    return $?
}
export -f bash_source_加载



## 好不好 \\
## Hǎo bù hǎo :: Good / Not Good
## is_file readable?
# n0t_file_📁_好不好 result:
#   0 : file is okay
#   1 : file is NOT okay
## if passed two or more files, will try all.
function n0ta_xfile_📁_好不好() {
    local args=("$@")
    xfile=${args[0]}
    if [ $# -gt 1 ] ; then
        # more than one file. try many
        while [ ! -x "$xfile" ] && [ "$#" -gt 1 ] ; do
            shift
            xfile=$1
        done
    fi

    if [ ! -f "$xfile" ] ; then
        log_📢_记录 "👽:不支持 $xfile is both required AND missing. 👽:非常要!"
        return 0
elif [ ! -x "$xfile" ] ; then
        log_📢_记录 "👽:不支持 $xfile is not executable. 👽:非常要!"
        return 0
    else

        #if ! has_fn_✅_存在 "toml_get" ; then
        #    :   # toml_get doesn't exist.
        #elif [[ $( toml_get "b00t" "has.$xfile" ) = "" ]] ; then
        #    log_📢_记录 "👍 $xfile"
        #    # toml_set "b00t" "has.sudo" $( yyyymmdd )
        #fi
        return 1
    fi
}
## 好不好 //

## future artificat,
function selectEditVSCode_experiment() {
    filename=$1
    # select file
    selectedFile="${ fzf $filename }"
    code -w $selectedFile
}



### - -   is_WSLv2_🐧💙🪟v2   - - \\
## Microsoft Windows Linux Subsystem II  WSL2
## 🤓 https://docs.microsoft.com/en-us/windows/wsl/install-win10
#
function is_WSLv2_🐧💙🪟v2() {
    return `cat /proc/version | grep -c "microsoft-standard-WSL2"`
}
### - -  ..  - - //


# Check if running in Claude Code environment
# Returns 0 (success) if CLAUDECODE=1, 1 (failure) otherwise
# Usage: if is_claudecode; then echo "In Claude Code"; fi
function is_claudecode() {
    [[ "${CLAUDECODE:-}" == "1" ]]
}

# Check if we should skip verbose output (motd, etc.)
# Returns 0 (success) to skip output, 1 (failure) for normal output
# Usage: if tokemoji_下文; then return; fi  # skip verbose output
function tokemoji_下文() {
    # this mode cuts down superfulous output
    if is_claudecode; then
        log_📢_记录 "🎆 hi Claude code!  🥾 b00t() ready!"
        return 0  # true - skip outpu
    fi
    # Add other criteria for skipping output here in the future
    return 1  # false - show normal output
}


# Check if running in VS Code integrated terminal
# Returns 0 (success) if VSCODE_GIT_IPC_HANDLE is set, 1 (failure) otherwise
# Usage: if is_vscode_shell; then echo "In VS Code terminal"; fi
function is_vscode_shell() {
    [[ -n "${VSCODE_GIT_IPC_HANDLE:-}" ]]
}

if is_vscode_shell; then
    log_📢_记录 "🥾💻 hi VS Code! running b00t-cli"
    b00t-cli vscode

fi



# 🍰 https://stackoverflow.com/questions/3963716/how-to-manually-expand-a-special-variable-ex-tilde-in-bash/29310477#29310477
# converts string ~/.b00t to actual path
# usage: path=$(expandPath '~/hello')
function expandPath() {
  local path
  local -a pathElements resultPathElements
  IFS=':' read -r -a pathElements <<<"$1"
  : "${pathElements[@]}"
  for path in "${pathElements[@]}"; do
    : "$path"
    case $path in
      "~+"/*)
        path=$PWD/${path#"~+/"}
        ;;
      "~-"/*)
        path=$OLDPWD/${path#"~-/"}
        ;;
      "~"/*)
        path=$HOME/${path#"~/"}
        ;;
      "~"*)
        username=${path%%/*}
        username=${username#"~"}
        IFS=: read -r _ _ _ _ _ homedir _ < <(getent passwd "$username")
        if [[ $path = */* ]]; then
          path=${homedir}/${path#*/}
        else
          path=$homedir
        fi
        ;;
    esac
    resultPathElements+=( "$path" )
  done
  local result
  printf -v result '%s:' "${resultPathElements[@]}"
  printf '%s\n' "${result%:}"
}


##* * * * * *\\
## 📽️️ Pr0J3ct1D
## uses inspiration
##* * * * * *//
function Pr0J3ct1D {
    local wordCount=$( cat $_b00t_INSPIRATION_FILE | jq '. | length' )
    # echo "wordCount: $wordCount"

    local word1=$( rand0 $wordCount )
    # echo "word1: $word1"
    local wordOne=$( cat $_b00t_INSPIRATION_FILE | jq ".[${word1}].word" -r )
    local word2=$( rand0 $wordCount )
    # echo "word2: $word2"
    local wordTwo=$( cat $_b00t_INSPIRATION_FILE | jq ".[${word2}].word" -r )
    local result="${wordOne}_${wordTwo}"

    ## todo: substitute
    if [ $( rand0 10 ) -lt 5 ] ; then
        result=$( echo $result | sed 's/l/1/g' )
    fi

    if [ $( rand0 10 ) -lt 2 ] ; then
        result=$( echo $result | sed 's/o/0/g' )
elif [ $( rand0 10 ) -lt 2 ] ; then
        result=$( echo $result | sed 's/oo/00/g' )
    fi

    if [ $( rand0 10 ) -lt 2 ] ; then
        result=$( echo $result | sed 's/e/3/g' )
elif [ $( rand0 10 ) -lt 8 ] ; then
        result=$( echo $result | sed 's/ee/33/g' )
    fi

    # Todo: fix first letter is a number. naming issue.

    echo $result
    return 0
}



##* * * * * *\\
## generates a random number between 0 and \$1
# usage:
# rand0_result="$(rand0 100)"
# echo \$rand0_result

function rand0() {
    local args=("$@")
    local max=${args[0]}
    rand0=$( bc <<< "scale=2; $(printf '%d' $(( $RANDOM % $max)))" ) ;
    # rand0=$( echo $RANDOM % $max ) ;
    echo $rand0
}

##* * * * * *//
## checks to see if an alias has been defined.
#  if is_n0t_aliased "az" ; then echo "true - not aliased!"; else echo "false"; fi
function is_n0t_aliased() {
    local args=("$@")
    local hasAlias=${args[0]}
    local exists=$(alias -p | grep "alias $hasAlias=")
    # echo "exists: $exists"
    if [ -z "$exists" ] ; then
        # 🙄 exists: alias fd='/usr/bin/fdfind'
        return 0;  # "true", unix success
    else
        return 1;  #  "false", unix error
    fi
}


##
## A pretty introduction to the system.
##
function motd() {
    # count motd's
    # 🍰 https://unix.stackexchange.com/questions/485221/read-lines-into-array-one-element-per-line-using-bash
    SED_PATH=$(whereis -b sed | cut -f 2 -d ' ')

    if is_n0t_aliased "fd" ; then
        # no fd, incomplete environemnt
        log_📢_记录 "🥾🍒 No fd alias, incomplete environment"
        if [ ! -f "/tmp/motd.txt" ] ; then
            printf "\nb00t basic motd. generated %s\n\n" $(ymd_hms) > "/tmp/motd.txt"
        fi
        motdz=('/tmp/motd.txt')
    else
        readarray -t motdz < <(/usr/bin/fdfind .txt "$_B00T_Path/./ubuntu.🐧/etc/")
    fi
    local motdzQ=$( rand0 ${#motdz[@]} )
    # declare -p motdz

    local showWithCMD="/usr/bin/batcat"

    f=${motdz[motdzQ]}
    local motdWidth=$(awk 'length > max_length { max_length = length; longest_line = $0 } END { print max_length }' $f)
    # local motdWidth=$(cat "${motdz[motdzQ]}" | tail -n 1)
    local motdLength=$(cat $f | wc -l)

    local myWidth=$(tput cols)
    local myHeight=$(tput rows)
    if [ -z "$myHeight" ] ; then myHeight='🍒😑' ; fi
    log_📢_记录 "🥾🖥️ motd .cols: $motdWidth  .rows:$motdLength"
    log_📢_记录 "🤓🖥️ user .cols: $myWidth  .rows:$myHeight"

    if [ $motdWidth -gt "$myWidth" ] ; then
        echo "👽:太宽 bad motd. too wide."
        showWithCMD=""
elif [ $motdWidth -gt $(echo $myWidth - 13 | bc) ] ; then
        # bat needs +13 columns
        showWithCMD="cat"
    else
        # *auto*, full, plain, changes, header, grid, numbers, snip.
        showWithCMD="batcat --pager=never --style=plain --wrap character"
        if [ $(rand0 100) -gt 69 ] ; then
            showWithCMD="batcat --pager=never --wrap character"
        fi
    fi
    # if it's still too big, try again!

    ## sometimes, cat is nice!
    if [ -z "$showWithCMD" ] ; then
        echo "👽💩: 烂狗屎 cannot motd."
elif [ "$(rand0 10)" -gt 5 ] ; then
        showWithCMD="cat"
    fi

    local glitchCMDz=''
    if [ $(rand0 10) -gt 1 ] ; then
        glitchCMDz="$glitchCMDz | $SED_PATH 's/1/0/g' "
    fi
    #if [ $(rand0 10) -gt 5 ] ; then
    #    glitchCMDz=" | $SED_PATH 's/0/1/g' $glitchCMDz"
    #fi
    #if [ $(rand0 10) -gt 5 ] ; then
    #    glitchCMDz=" | $SED_PATH 's/8/🥾/g' $glitchCMDz"
    #fi

    #if [ $motdLength -gt $(echo $(tput rows) - 3 | bc) ] ; then
    #    showWithCMD="cat"
    #fi

    if [ -n "$showWithCMD" ] ; then
        motdTmpFile=$( mktemp "_b00t_.日$(ymd).一时XXXXXXXXXX.motd" )
        # echo "motdFile: $motdTmpFile"
        # echo $(rand0 10)
        ## glitch effects
        cp -v ${motdz[motdzQ]} $motdTmpFile
        if [ $(rand0 10) -gt 5 ] ; then
            $SED_PATH -i 's/1/0/g' $motdTmpFile
            $SED_PATH -i 's/8/🥾/g' $motdTmpFile
        fi
        if [ $(rand0 10) -gt 5 ] ; then
            $SED_PATH -i 's/\*/🥾/g' $motdTmpFile
            $SED_PATH -i 's/[\!\-\@]./😁/g' $motdTmpFile
        fi
        if [ $(rand0 10) -gt 5 ] ; then
            $SED_PATH -i 's/#/_/g' $motdTmpFile
            $SED_PATH -i 's/0/🐛/g' $motdTmpFile
        fi
        if [ $(rand0 10) -gt 5 ] ; then
            $SED_PATH -i 's/1/l/g' $motdTmpFile
            $SED_PATH -i 's/[\@l\#]/🐛/g' $motdTmpFile
        fi
        $showWithCMD $motdTmpFile
        /bin/rm -f $motdTmpFile
    fi

    # part of motd

    log_📢_记录 "lang: $LANG"
    log_📢_记录 "🥾📈 motd project stats, cleanup, tasks goes here. "
    local skunk_x=0
    if [ -d .git ]; then
        gh issue list
        skunk_x=$(git grep "🦨" | wc -l)
    elif [ -d "$HOME/.dotfiles/.git" ] ; then

        log_📢_记录 "🥾: found $HOME/.dotfiles/.git repo"
        # github client
        ##  if a .git dir exists, check for local issues, otheriwse list for ~/.dotfiles
        (cd "$HOME/.dotfiles" && gh issue list && local skunk_x=$(git grep "🦨" | wc -l) &&  log_📢_记录 "🦨: $skunk_x")
        SRC_VERSION=$(cd "$HOME/.dotfiles" && gh release view -R elasticdotventures/dotfiles --json tagName | jq -r .tagName)

        OUR_VERSION=$(_b00t_get_version "$HOME/.dotfiles")

        if [ "$SRC_VERSION" == "$OUR_VERSION" ]; then
            log_📢_记录 "🥾📈: is on latest: $OUR_VERSION  (local)"
        else
            log_📢_记录 "🥾📈: latest release: $SRC_VERSION | local: $OUR_VERSION"
        fi


    else
        log_📢_记录 "🥾🐙😔 no ~/.dotfiles/.git dir "`pwd`
    fi

    if [ "$skunk_x" -gt 0 ] ; then
        log_📢_记录 "🥾🐙🦨: found $skunk_x 🦨 issues in this repo"
    fi

}

if tokemoji_下文; then
    # Skip motd when in Claude Code or other quiet environments
    :  # no-op
elif [ "${container+}" == "docker" ] ; then
    motd
elif ! is_n0t_aliased fd ; then
    motd
else
    motd
fi





# export FZF_COMPLETION_OPTS='--border --info=inline'
if ! n0ta_xfile_📁_好不好 "/usr/bin/fdfind"  ; then
    # export FZF_DEFAULT_COMMAND="git ls-files --cached --others --exclude-standard | /usr/bin/fdfind --type f --type l $FD_OPTIONS"
    # export FZF_DEFAULT_OPTS="--no-mouse --height 50% -1 --reverse --multi --inline-info --preview=[[ \$file --mine{}) =~ binary ]] && echo {} is a binary file || (bat --style=numbers --color=always {} || cat {}) 2> /dev/null | head -300' --preview-window='right:hidden:wrap' --bind'f3:execute(bat --style=numbers {} || less -f {}),f2:toggle-preview,ctrl-d:half-page-down,ctrl-u:half-page-up,ctrl-a:select-all+accept,ctrl-y:execute-silent(echo {+} | pbcopy)'"
    # from video: https://www.youtube.com/watch?v=qgG5Jhi_Els
    export FZF_DEFAULT_COMMAND="/usr/bin/fdfind --type f"
fi



## this must be sourced at the end
## REMOVING TOML host storage .. keep for
#source ~/.dotfiles/_b00t_/bash.🔨/_/toml-cli.sh
#toml_init





##
## there's time we need to know reliably if we can run SUDO
##
function has_sudo() {
    SUDO_CMD="/usr/bin/sudo"

    ## 🦨 TODO: ask for consent to run sudo?

    if [ "$EUID" -eq 0 ] ; then
        # r00t doesn't require sudo
        # https://stackoverflow.com/questions/18215973/how-to-check-if-running-as-root-in-a-bash-script
        log_📢_记录 "👹 please don't b00t as r00t"
        SUDO_CMD=""
elif [ -f "./dockerenv" ] ; then
        # https://stackoverflow.com/questions/23513045/how-to-check-if-a-process-is-running-inside-docker-container#:~:text=To%20check%20inside%20a%20Docker,%2Fproc%2F1%2Fcgroup%20.
        log_📢_记录 "🐳😁 found DOCKER"
elif [ -f "$SUDO_CMD" ] ; then
        #if [[ -z $( toml_get "b00t" "has.sudo" )  ]] ; then
        log_📢_记录 "🥳 found sudo"
            # toml_set "b00t" "has.sudo" `ymd_hms`
        #fi
    else
        log_📢_记录 "🐭 missed SUDO, try running _b00t_ inside docker."
        SUDO_CMD=""
    fi
    export SUDO_CMD
}

is_b00table() {
    # a simple set of checks to make sure b00t required tools are available.
    [ -n "${_B00T_MISSING_TOOLS_:-}" ] && return 0 || return 1
}

if ! is_b00table ; then
    has_sudo
fi


#############################
###
# 🍰 https://superuser.com/questions/427318/test-if-a-package-is-installed-in-apt
#if debInst "$1"; then
#    printf 'Why yes, the package %s _is_ installed!\\n' "$1"
#else
#    printf 'I regret to inform you that the package %s is not currently installed.\\n' "$1"
#fi
function debInst() {
    dpkg-query -Wf'${db:Status-abbrev}' "$1" 2>/dev/null | grep -q '^i'
}

# if [ ! is_b00table ] ; then
#     log_📢_记录 "🥾😭 missing tools"
# elif debInst "moreutils" ; then
#     # Only show moreutils once.
#     value=$( toml_get "b00t" "has.moreutils" )
#     value=${value:-0}  # If value is empty, default to 0
#     if [ "$value" -eq "0" ] ; then
#         log_📢_记录 "👍 debian moreutils is installed!"
#         # toml_set "b00t" "has.moreutils" "$(yyyymmdd)"
#     fi
# else
#     log_📢_记录 "😲 install moreutils (required)"
#     $SUDO_CMD apt-get install -y moreutils
# fi


# 60秒 Miǎo seconds
# 3分钟 Fēnzhōng minutes
# 3小时 Xiǎoshí seconds

# export _b00t_JS0N_filepath=$(expandPath "~/.b00t/config.json")
#function jqAddConfigValue () {
#    echo '{ "names": ["Marie", "Sophie"] }' |\\
#    jq '.names |= .+ [
#        "Natalie"
#    ]'
#}

# 🍰 https://lzone.de/cheat-sheet/jq
#function jqSetConfigValue () {

##
export _user="$(id -u -n)"
export _uid="$(id -u)"
echo "🙇‍♂️ \$_user: $_user  \$_uid : $_uid"
set +o nounset
#set +a  # turn off export all (breaks bash autocomplete)

##
if [ -f "__b00t__.sh" ]; then
    echo "__b00t__.sh"
    source "__b00t__.sh"
fi

# b00t command aliases and functions
if [ -x ~/.cargo/bin/b00t-cli ]; then
    # b00t-cli is installed, use it.
    export PATH="$PATH:~/.cargo/bin"
    alias b00t='~/.cargo/bin/b00t-cli'
else
    # b00t-cli is not installed, try to install it.
    # if [ -f "$HOME/.dotfiles/_b00t_/bash.🔨/_/b00t-cli.sh" ]; then
    #     source "$HOME/.dotfiles/_b00t_/bash.🔨/_/b00t-cli.sh"
    # else
        echo "🥾💔 b00t-cli not found, please install it."
    # fi
fi

# test for alias b00t exists

function _b00t_check() {
    local outdated_found=0
    local toml_dir="$HOME/.dotfiles/_b00t_"

    # Get all .toml files, sort them alphabetically, and iterate
    for toml_file in $(find "$toml_dir" -maxdepth 1 -name "*.toml" | sort); do
        local command_name=$(basename "$toml_file" .toml)
        b00t . "$command_name"
        local exit_code=$?
        if [ $exit_code -eq 1 ] || [ $exit_code -eq 2 ]; then
            outdated_found=1
        fi
    done

    if [ $outdated_found -ne 0 ]; then
        echo "🥾💡 Some tools are out of date or missing. Run 'b00t up' to update them."
    fi
}

# Run _b00t_check automatically when .bashrc is sourced
# check b00t bash alias (was it created, then it exists)
if ! is_n0t_aliased "b00t" ; then
    _b00t_check
fi

# alias b00t-check=_b00t_check
# alias b00t-up="b00t up"
