<?php require_once  __SITE_PATH . "/view/_header.php";?>
<div class=flex-container>

<aside id="chat_ranking1">
<h3 id=chat-heading>Chat</h3>
<div id="chat"></div>
<br />
<input type="text" id="txt" class="input">
<button id="btn" class="button">Send</button>
<br><br>
<div id="game_flow">
    Time remaining:
    <span id="timer"></span>
    <br>
    Look for:
    <span id="token"></span>
</div>
<br><br>
</aside>

<script src="https://ajax.googleapis.com/ajax/libs/jquery/3.3.1/jquery.js"></script>
<?php $_SESSION["game"]->draw_board() ?>
<br>

<aside id="chat_ranking2">
<h3>Licitations</h3>
<div id="ranking"></div>
<h3>Score</h3>
<div id="result"></div>
<br><br>
</aside>
</div>
<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <input type="submit" value="Back" id="gameplay_button" class="button"> 
</form>


<script src= <?php echo __SITE_URL . "/js/robot_movement.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/revert_moves.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/utils.js"; ?>></script>
<script src= <?php echo __SITE_URL . "/js/gameplay.js"; ?>></script>

<?php require_once __SITE_PATH . "/view/_footer.php";?>