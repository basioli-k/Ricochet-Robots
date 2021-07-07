
<?php require_once  __SITE_PATH . "/view/_header.php";?>

<form action="<?php echo __SITE_URL; ?>/index.php?rt=stats" method="post">

<!-- <ul class="radio_stat">
    <li class="radio_stat_element">
        <label for="sort_by"> Games won </label>
        <input type="radio" name="sort_by" value="games_won" checked>
    </li>
    <li class="radio_stat_element">
    <label for="sort_by"> Games played </label>
        <input type="radio" name="sort_by" value="games_played">   
    </li>
    <li class="radio_stat_element">
        <label for="sort_by"> Tokens won </label>
        <input type="radio" name="sort_by" value="tokens_won">
    </li>
</ul>
<input type="submit" value="Sort"> -->
<div class="btn_group">
    <button name="btn_stats" class="btn_stats" value="games_won">Games won</button>
    <button name="btn_stats" class="btn_stats" value="games_played">Games played</button>
    <button name="btn_stats" class="btn_stats" value="tokens_won">Tokens won</button>
</div>
</form>
<br>
<table class= "stat_table">
    <tr class="stat_row">
        <th class="stat_row_head"> Username </th>
        <th class="stat_row_head"> <?php echo $column_name; ?> </th>
    </tr>
    <?php
        foreach($players as $player){
            echo "<tr class=\"stat_row\"> <td class=\"stat_row_data\">" . $player->username . "</td>" . "<td class=\"stat_row_data\">" . $player->$sort_by . "</td>" . "</tr>";
        }
    ?>
</table>
<br>
<form action=" <?php echo __SITE_URL ?>/index.php" method="get">
    <button class="btn_stats" id="back" value="Back">Back</button>
</form>


<?php require_once __SITE_PATH . "/view/_footer.php";?>